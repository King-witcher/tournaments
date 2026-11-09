"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { FormModal } from "@/shared/ui/form-modal";
import { TextInput } from "@/shared/ui/text-input";
import { parseInviteInput } from "../lib/invite-input";

type Props = {
  /** O botão que abriu, para devolver o foco ao cancelar. */
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
};

/**
 * O campo "Link ou código". Interpreta o texto (`parseInviteInput`) e, se for
 * convite, confere antes de sair daqui: não existindo, o erro fica no campo
 * com o que foi digitado; existindo, leva à página da prévia e do aceite.
 * Outro endereço do site só leva a ele.
 */
export function JoinWithCodeModal({ trigger, onClose }: Props) {
  const t = useTranslations("Invite");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [value, setValue] = useState("");
  // O que não é da API ou tem texto próprio aqui; vale mais que o `error`.
  const [problem, setProblem] = useState<string>();
  const shown = pending ? undefined : (problem ?? error);

  const submit = () => {
    setProblem(undefined);
    const input = parseInviteInput(value, window.location.host);
    if (input.kind === "invalid") {
      setProblem(t("join.invalid"));
      return;
    }
    if (input.kind === "path") {
      router.push(input.path);
      onClose();
      return;
    }
    const { token } = input;
    run(
      () => api.GET("/invites/{token}", { params: { path: { token } } }),
      () => router.push(`/invite/${token}`),
      (code) => {
        if (code === "invite.not_found") setProblem(t("join.notFound"));
        if (code === "tenant.deletion_scheduled") {
          setProblem(t("deletionScheduled"));
        }
      },
    );
  };

  return (
    <FormModal
      title={t("join.open")}
      submitLabel={t("join.submit")}
      onSubmit={submit}
      onClose={onClose}
      pending={pending}
      submitDisabled={!value.trim()}
      returnFocusTo={trigger}
    >
      <TextInput
        labelText={t("join.field")}
        value={value}
        onChange={(event) => setValue(event.target.value)}
        autoCapitalize="off"
        autoCorrect="off"
        autoComplete="off"
        spellCheck={false}
        invalid={!!shown}
        invalidText={shown}
      />
    </FormModal>
  );
}
