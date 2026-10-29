import { Column, Grid, Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { RatingSettingsSection } from "@/features/ratings/components/rating-settings-section";
import { getTenant } from "@/features/tenants/server";
import { Section } from "@/shared/ui/section";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { LeaveTenantSection } from "./leave-tenant-section";
import { NicknameForm } from "./nickname-form";
import { TransferOwnershipSection } from "./transfer-ownership-section";

/** O bloco de sair ou transferir, que recebe o foco quando a posse muda. */
const MEMBERSHIP_ID = "membership";

/**
 * A aba Configurações: o apelido de quem vê, o rating da comunidade (só o
 * owner) e, embaixo, sair da comunidade (ou, para o owner, transferir a posse
 * antes). Na coluna de formulário (8
 * das 16 colunas a partir do `lg`). Fica em `<Suspense>`.
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
          <div id={MEMBERSHIP_ID} tabIndex={-1}>
            {tenant.role === "owner" ? (
              <TransferOwnershipSection slug={slug} focusId={MEMBERSHIP_ID} />
            ) : (
              <LeaveTenantSection slug={slug} />
            )}
          </div>
        </Stack>
      </Column>
    </Grid>
  );
}
