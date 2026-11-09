import { getTranslations } from "next-intl/server";

import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { Stack } from "@/shared/ui/carbon/stack";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { DeleteTenantButton } from "./delete-tenant-button";

type Props = {
  tenant: components["schemas"]["MyTenant"];
  /** O bloco que recebe o foco quando o botão troca. */
  focusId: string;
};

/**
 * Para o dono, nas configurações: excluir a comunidade, que sai na hora sem
 * torneio começado e, com histórico, em 7 dias; agendada, a data e "Cancelar
 * exclusão"; com torneio em andamento, o motivo de não dar. Sem saber o que a
 * exclusão faria (a API falhou), o bloco some em vez de prometer o errado.
 */
export async function DeleteTenantSection({ tenant, focusId }: Props) {
  const due = tenant.deletion_scheduled_for;
  const api = await serverApi();
  const [t, effect] = await Promise.all([
    getTranslations("Tenant.deletion"),
    due
      ? undefined
      : api
          .GET("/tenants/{slug}/deletion", {
            params: { path: { slug: tenant.slug } },
          })
          .then(({ data }) => data?.effect)
          .catch(() => undefined),
  ]);
  if (!due && !effect) return null;
  const blockedReason =
    effect === "blocked"
      ? await apiErrorMessage("tenant.tournament_in_progress")
      : undefined;

  return (
    <Section
      level={3}
      title={t("title")}
      description={effect === "scheduled" ? t("descriptionLater") : undefined}
    >
      <Stack gap={5}>
        {due && (
          <Notice kind="warning" title={t("scheduledTitle")}>
            {/* Num span: o conteúdo do aviso é flex, e a data quebraria a frase. */}
            <span>
              {t.rich("scheduled", {
                date: () => <LocalDateTime value={due} />,
              })}
            </span>
          </Notice>
        )}
        <DeleteTenantButton
          slug={tenant.slug}
          scheduled={due != null}
          now={effect === "immediate"}
          blockedReason={blockedReason}
          focusId={focusId}
        />
      </Stack>
    </Section>
  );
}
