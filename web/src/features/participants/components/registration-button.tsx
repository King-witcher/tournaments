"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";

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

  return (
    <div className="flex flex-col items-start gap-1">
      {registered ? (
        <ConfirmButton
          pending={pending}
          question={t("withdrawConfirm")}
          onConfirm={() =>
            run(
              () => api.DELETE(path, { params }),
              () => router.refresh(),
            )
          }
        >
          {t("withdraw")}
        </ConfirmButton>
      ) : (
        <Button
          pending={pending}
          onClick={() =>
            run(
              () => api.POST(path, { params }),
              () => router.refresh(),
            )
          }
        >
          {t("register")}
        </Button>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
