"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";

const REGISTRATION_ID = "registration-action";

type Props = {
  tenant: string;
  slug: string;
  /** Inscrito: o botão cancela a inscrição. */
  registered: boolean;
};

/** Inscreve quem vê no torneio, ou cancela a inscrição (depois de confirmar: a vaga pode ir para outro). */
export function RegistrationButton({ tenant, slug, registered }: Props) {
  const t = useTranslations("Participants.registration");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { slug: tenant, tournament: slug } };
  const path = "/tenants/{slug}/tournaments/{tournament}/registration";
  // Inscrever troca o botão pelo de cancelar (e vice-versa) no refresh: o
  // foco vai para o bloco novo, em vez de cair no começo da página.
  const done = () => {
    router.refresh();
    focusIfLost(REGISTRATION_ID);
  };

  return (
    <div
      id={REGISTRATION_ID}
      tabIndex={-1}
      className="flex flex-col items-start gap-1"
    >
      {registered ? (
        <ConfirmButton
          pending={pending}
          question={t("withdrawConfirm")}
          onConfirm={() => run(() => api.DELETE(path, { params }), done)}
        >
          {t("withdraw")}
        </ConfirmButton>
      ) : (
        <Button
          pending={pending}
          onClick={() => run(() => api.POST(path, { params }), done)}
        >
          {t("register")}
        </Button>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
