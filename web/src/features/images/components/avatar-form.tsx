"use client";

import { useTranslations } from "next-intl";
import { useState, useTransition } from "react";

import { useRouter } from "@/i18n/navigation";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { LeaveGuard } from "@/shared/ui/leave-guard";
import { Notice } from "@/shared/ui/notice";
import { useImageDraft } from "../hooks/use-image-draft";
import { ImageField } from "./image-field";

const TARGET = { kind: "avatar" } as const;

type Props = {
  /** A foto salva; nula sem ela. */
  url: string | null | undefined;
  maxBytes: number;
};

/**
 * A foto de perfil num formulário: a foto pendente (`useImageDraft`) e o Salvar
 * dela, como um campo, e nada vai para a API antes dele. O Salvar só libera com
 * uma mudança; sair com ela pergunta antes (`LeaveGuard`).
 */
export function AvatarForm({ url, maxBytes }: Props) {
  const t = useTranslations("Images.avatar");
  const router = useRouter();
  const draft = useImageDraft(TARGET);
  const [pending, startTransition] = useTransition();
  const [saved, setSaved] = useState(false);

  const save = () =>
    startTransition(async () => {
      setSaved(false);
      if (await draft.apply()) {
        setSaved(true);
        router.refresh();
      }
    });

  return (
    <Stack gap={6}>
      <ImageField
        draft={draft}
        url={url}
        maxBytes={maxBytes}
        saving={pending}
      />
      <Form
        onSubmit={(event) => {
          event.preventDefault();
          if (!pending && draft.dirty) save();
        }}
      >
        <Stack gap={6}>
          {saved && !draft.dirty && (
            <Notice kind="success">{t("saved")}</Notice>
          )}
          <FormActions pending={pending} disabled={!draft.dirty} />
        </Stack>
      </Form>
      <LeaveGuard unsaved={draft.dirty} path="/account" />
    </Stack>
  );
}
