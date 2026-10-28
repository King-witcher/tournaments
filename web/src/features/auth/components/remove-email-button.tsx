"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { LOGIN_SECTION } from "../lib/login-sections";

type Props = {
  /** A conta tem senha, que só se recupera pelo email: o botão vem travado, com o motivo. */
  blocked: boolean;
};

/**
 * Tira o email da conta, depois do `ConfirmButton` dizer o que se perde. Com
 * senha na conta, travado e com o motivo embaixo (a API também recusa). O
 * botão some no refresh, e o foco vai para o bloco do email.
 */
export function RemoveEmailButton({ blocked }: Props) {
  const t = useTranslations("Account.email");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote note={blocked && t("removeBlocked")} error={error}>
      <ConfirmButton
        size="md"
        question={t("removeConfirm")}
        pending={pending}
        disabled={blocked}
        onConfirm={() =>
          run(
            () => api.DELETE("/auth/email"),
            () => {
              router.refresh();
              focusIfLost(LOGIN_SECTION.email);
            },
          )
        }
      >
        {t("remove")}
      </ConfirmButton>
    </ActionWithNote>
  );
}
