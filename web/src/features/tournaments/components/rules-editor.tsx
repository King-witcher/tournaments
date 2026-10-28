"use client";

import { Edit } from "@carbon/icons-react";
import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { startTransition, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { EmptyState } from "@/shared/ui/empty-state";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { TextArea } from "@/shared/ui/text-area";
import { UserText } from "@/shared/ui/user-text";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  tournament: string;
  /** O idioma do texto mostrado (o original ou o da tradução). */
  locale: Schemas["Locale"];
  rules: string;
  /** O texto é de uma tradução: salvar a regrava com o nome e a descrição dela. */
  translation?: Schemas["TournamentTranslation"];
};

/**
 * A aba Regras para a organização: o texto com "Editar regras" em cima (ou,
 * sem regras, o convite a escrever), que troca o texto por um campo com
 * Salvar e Cancelar ali mesmo. Muda o texto que a pessoa está lendo: o da
 * tradução no idioma dela, se tem, ou o original. Salvo, o editor fecha com
 * o texto novo e o foco volta ao botão.
 */
export function RulesEditor({
  tenant,
  tournament,
  locale,
  rules,
  translation,
}: Props) {
  const t = useTranslations("Tournament.rulesEditor");
  const tForm = useTranslations("TournamentForm");
  const tLocales = useTranslations("Locales");
  const router = useRouter();
  const editId = useId();
  const fieldId = useId();
  const { run, pending, error } = useApiAction();
  const [editing, setEditing] = useState(false);
  const [text, setText] = useState(rules);

  const open = () => {
    setText(rules);
    setEditing(true);
    // Abriu pelo botão: quem clicou quer escrever.
    requestAnimationFrame(() => document.getElementById(fieldId)?.focus());
  };
  const close = () => {
    setEditing(false);
    focusIfLost(editId);
  };
  const save = () => {
    const path = { slug: tenant, tournament };
    run(
      () =>
        translation
          ? api.PUT(
              "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
              {
                params: { path: { ...path, locale } },
                body: {
                  name: translation.name,
                  description: translation.description,
                  rules: text,
                },
              },
            )
          : api.PATCH("/tenants/{slug}/tournaments/{tournament}", {
              params: { path },
              body: { rules: text },
            }),
      // Fecha junto com a recarga, para não mostrar o texto antigo antes.
      () =>
        startTransition(() => {
          router.refresh();
          close();
        }),
    );
  };

  if (!editing) {
    const edit = (
      <Button id={editId} kind="tertiary" renderIcon={Edit} onClick={open}>
        {rules ? t("edit") : t("write")}
      </Button>
    );
    return rules ? (
      <Stack gap={6}>
        <div>{edit}</div>
        <UserText>{rules}</UserText>
      </Stack>
    ) : (
      <EmptyState action={edit}>{t("empty")}</EmptyState>
    );
  }

  return (
    <Form
      onSubmit={(event) => {
        event.preventDefault();
        if (!pending) save();
      }}
    >
      <Stack gap={6}>
        <TextArea
          labelText={t(translation ? "labelTranslation" : "labelOriginal", {
            language: tLocales(locale),
          })}
          helperText={tForm("plainTextHint")}
          rows={16}
          maxLength={20000}
          id={fieldId}
          lang={locale}
          value={text}
          onChange={(event) => setText(event.target.value)}
        />
        {error && <FormError>{error}</FormError>}
        <FormActions pending={pending} onCancel={close} />
      </Stack>
    </Form>
  );
}
