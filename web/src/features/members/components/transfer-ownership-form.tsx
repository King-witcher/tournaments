"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { ErrorText } from "@/shared/ui/error-text";
import { Select } from "@/shared/ui/select";

type Props = {
  slug: string;
  /** Membros que podem receber a posse (todos menos o owner). */
  candidates: { userId: number; nickname: string }[];
  /** O bloco que recebe o foco depois (a seção some: quem vê deixou de ser owner). */
  focusId: string;
};

/**
 * Escolher o novo owner e confirmar no Modal, digitando o endereço da
 * comunidade. Sem escolha, o botão aponta o campo em vez de abrir o Modal.
 */
export function TransferOwnershipForm({ slug, candidates, focusId }: Props) {
  const t = useTranslations("Tenant.transfer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const trigger = useRef<HTMLButtonElement>(null);
  const [userId, setUserId] = useState("");
  const [missing, setMissing] = useState(false);
  const [open, setOpen] = useState(false);
  const chosen = candidates.find((c) => String(c.userId) === userId);

  const close = () => {
    setOpen(false);
    requestAnimationFrame(() => trigger.current?.focus());
  };

  return (
    <Stack gap={6}>
      <Select
        labelText={t("member")}
        value={userId}
        onChange={(event) => {
          setUserId(event.target.value);
          setMissing(false);
        }}
        invalid={missing}
        invalidText={t("chooseFirst")}
      >
        <option value="">{t("choose")}</option>
        {candidates.map((c) => (
          <option key={c.userId} value={c.userId}>
            {c.nickname}
          </option>
        ))}
      </Select>
      <Stack gap={3}>
        <div>
          <Button
            ref={trigger}
            kind="danger--tertiary"
            pending={pending}
            onClick={() => (chosen ? setOpen(true) : setMissing(true))}
          >
            {t("submit")}
          </Button>
        </div>
        {error && <ErrorText>{error}</ErrorText>}
      </Stack>
      <ConfirmModal
        open={open && chosen !== undefined}
        onClose={close}
        onConfirm={() => {
          close();
          if (!chosen) return;
          run(
            () =>
              api.POST("/tenants/{slug}/transfer-ownership", {
                params: { path: { slug } },
                body: { user_id: chosen.userId, confirm_slug: slug },
              }),
            () => {
              setUserId("");
              router.refresh();
              focusIfLost(focusId);
            },
          );
        }}
        title={t("submit")}
        confirmLabel={t("submit")}
        destructive
        slug={slug}
      >
        {t("confirm", { name: chosen?.nickname ?? "" })}
      </ConfirmModal>
    </Stack>
  );
}
