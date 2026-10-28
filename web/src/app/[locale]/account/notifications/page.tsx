import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { PushSettings } from "@/features/push/components/push-settings";
import { SoundSettings } from "@/features/realtime/components/sound-settings";
import { Section } from "@/shared/ui/section";

/**
 * Aba Avisos da conta: os avisos com o site fechado (Web Push) e o som dos
 * avisos com o site aberto, os dois deste navegador. Não busca dado no
 * servidor (o estado é do navegador), por isso sem `loading.tsx`.
 */
export default async function AccountNotificationsPage() {
  const t = await getTranslations("Notifications");
  return (
    <Stack gap={7}>
      <Section title={t("push.title")} description={t("push.description")}>
        <PushSettings />
      </Section>
      <Section title={t("sound.title")} description={t("sound.description")}>
        <SoundSettings />
      </Section>
    </Stack>
  );
}
