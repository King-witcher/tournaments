"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = {
  tenant: string;
  slug: string;
  /** Inscrito: o botão cancela a inscrição. */
  registered: boolean;
};

/** Inscreve quem vê no torneio, ou cancela a inscrição. */
export function RegistrationButton({ tenant, slug, registered }: Props) {
  const t = useTranslations("Participants.registration");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { slug: tenant, tournament: slug } };
  const path = "/tenants/{slug}/tournaments/{tournament}/registration";

  return (
    <div className="flex flex-col items-start gap-1">
      <Button
        variant={registered ? "outline" : "default"}
        size={registered ? "sm" : "default"}
        disabled={pending}
        onClick={() =>
          run(
            () =>
              registered
                ? api.DELETE(path, { params })
                : api.POST(path, { params }),
            () => router.refresh(),
          )
        }
      >
        {registered ? t("withdraw") : t("register")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
