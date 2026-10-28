"use client";

import { useTranslations } from "next-intl";

import { forgetPush } from "@/features/push/lib/forget-push";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { UserPanel } from "@/shared/ui/user-panel";

type Props = {
  label: string;
  name: string;
  avatarSrc?: string | null;
};

/**
 * O painel de quem está logado: Conta e Sair. Sair desliga antes os avisos
 * deste navegador (`forgetPush`) e volta para o início: quase toda página é
 * só para membros e, sem sessão, só mostraria "entre para ver".
 */
export function UserMenuPanel({ label, name, avatarSrc }: Props) {
  const t = useTranslations("Auth");
  const router = useRouter();
  const { run, pending } = useApiAction();

  return (
    <UserPanel
      label={label}
      name={name}
      avatarSrc={avatarSrc}
      items={[
        { label: t("account"), href: "/account" },
        {
          label: t("logout"),
          pending,
          onSelect: () =>
            run(
              async () => {
                await forgetPush();
                return api.POST("/auth/logout");
              },
              () => {
                router.replace("/");
                router.refresh();
              },
            ),
        },
      ]}
    />
  );
}
