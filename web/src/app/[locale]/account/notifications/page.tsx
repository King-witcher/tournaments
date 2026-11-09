import { getTranslations } from "next-intl/server";
import { AccountColumns } from "@/features/auth/components/account-columns";
import { PushSettings } from "@/features/push/components/push-settings";
import { SoundSettings } from "@/features/realtime/components/sound-settings";
import { PageHeader } from "@/shared/ui/page-header";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";

/**
 * As Notificações da conta, lado a lado: as com o site fechado (Web Push) e
 * o som delas com o site aberto, os dois deste navegador. Não busca dado no
 * servidor (o estado é do navegador); o `loading.tsx` é só para a navegação
 * não mostrar o esqueleto do Perfil.
 */
export default async function AccountNotificationsPage() {
  const [t, tAccount] = await Promise.all([
    getTranslations("Notifications"),
    getTranslations("Account"),
  ]);
  return (
    <>
      <PageHeader
        breadcrumbs={[{ href: "/account", label: tAccount("title") }]}
        title={tAccount("tabs.notifications")}
      />
      <AccountColumns
        start={
          <Panel>
            <Section
              title={t("push.title")}
              description={t("push.description")}
            >
              <PushSettings />
            </Section>
          </Panel>
        }
        end={
          <Panel>
            <Section
              title={t("sound.title")}
              description={t("sound.description")}
            >
              <SoundSettings />
            </Section>
          </Panel>
        }
      />
    </>
  );
}
