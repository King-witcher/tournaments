import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
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
 * Para o dono, nas configurações: excluir a comunidade, com o prazo e o que
 * acontece; agendada, a data e "Cancelar exclusão".
 */
export async function DeleteTenantSection({ tenant, focusId }: Props) {
  const t = await getTranslations("Tenant.deletion");
  const due = tenant.deletion_scheduled_for;

  return (
    <Section
      level={3}
      title={t("title")}
      description={due ? undefined : t("description")}
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
          focusId={focusId}
        />
      </Stack>
    </Section>
  );
}
