import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import { Section } from "@/shared/ui/section";
import { getImageSettings } from "../server";
import { AvatarField } from "./avatar-field";

/**
 * A foto de perfil, na aba Perfil da conta. Sem o bucket público na API,
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
    <Section
      title={t("title")}
      description={discord ? t("discord") : undefined}
    >
      <AvatarField url={user.photo_url} maxBytes={settings.max_bytes} />
    </Section>
  );
}
