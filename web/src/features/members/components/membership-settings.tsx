import { Column, Grid, Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RatingSettingsSection } from "@/features/ratings/components/rating-settings-section";
import { getTenant } from "@/features/tenants/server";
import { Section } from "@/shared/ui/section";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { NicknameForm } from "./nickname-form";

/**
 * A aba Configurações: o apelido de quem vê e o rating da comunidade (só o
 * owner). A zona de perigo (`CommunityDangerZone`) vem por último na página,
 * depois do Discord. Na coluna de formulário (8 das 16 colunas a partir do
 * `lg`). Fica em `<Suspense>`.
 */
export async function MembershipSettings({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.nickname"),
  ]);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <Stack gap={9}>
          <Section title={t("title")}>
            <NicknameForm slug={slug} nickname={tenant.nickname} />
          </Section>
          {tenant.role === "owner" && (
            <Suspense fallback={<SectionSkeleton rows={4} />}>
              <RatingSettingsSection slug={slug} />
            </Suspense>
          )}
        </Stack>
      </Column>
    </Grid>
  );
}
