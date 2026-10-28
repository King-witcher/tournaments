"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = { slug: string; userId: number; nickname: string };

/**
 * Remove o membro, depois de confirmar o que acontece com ele (o mesmo que
 * sair: inscrições abertas, W.O. nos torneios em andamento, convites).
 */
export function RemoveMemberButton({ slug, userId, nickname }: Props) {
  const t = useTranslations("Tenant.members");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-end">
      <Button
        variant="link"
        disabled={pending}
        className="text-red-600 dark:text-red-400"
        onClick={() => {
          if (!window.confirm(t("removeConfirm", { name: nickname }))) return;
          run(
            () =>
              api.DELETE("/tenants/{slug}/members/{user_id}", {
                params: { path: { slug, user_id: userId } },
              }),
            () => router.refresh(),
          );
        }}
      >
        {t("remove")}
      </Button>
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
