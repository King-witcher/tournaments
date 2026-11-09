import { getTranslations } from "next-intl/server";
import { getCurrentUser } from "@/features/auth/server";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { getImageSettings } from "../server";
import { AvatarForm } from "./avatar-form";

/**
 * A foto de perfil, no Perfil da conta. Sem o bucket público na API,
 * nada. Fica em `<Suspense>`.
 */
export async function AvatarSection() {
  const [user, settings, t] = await Promise.all([
    getCurrentUser(),
    getImageSettings(),
    getTranslations("Images.avatar"),
  ]);
  if (!user || !settings?.enabled) return null;
  // Sem foto, as telas mostram o avatar do Discord, que a área de soltar não
  // mostra.
  const discord = !user.photo_url && Boolean(user.avatar_url);

  return (
    <Panel>
      <Section
        title={t("title")}
        description={discord ? t("discord") : undefined}
      >
        <AvatarForm url={user.photo_url} maxBytes={settings.max_bytes} />
      </Section>
    </Panel>
  );
}
