import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { DiscordSection } from "@/features/discord/components/discord-section";
import { MembershipSettings } from "@/features/members/components/membership-settings";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/** Aba Configurações da comunidade: o que é de quem vê e, para o owner, o Discord. */
export default async function TenantSettingsPage({
  params,
}: PageProps<"/[locale]/[tenant]/settings">) {
  const t = await getTranslations("Tenant.settings");
  return (
    <Stack gap={9}>
      <Suspense
        fallback={
          <TabLoading label={t("loading")}>
            <SectionSkeleton rows={2} />
            <SectionSkeleton rows={1} />
          </TabLoading>
        }
      >
        {params.then(({ tenant }) => (
          <MembershipSettings slug={tenant} />
        ))}
      </Suspense>
      <Suspense fallback={null}>
        {params.then(({ tenant }) => (
          <DiscordSection slug={tenant} />
        ))}
      </Suspense>
    </Stack>
  );
}
