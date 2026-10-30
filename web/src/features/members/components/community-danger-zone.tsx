import { Column, Grid } from "@carbon/react";

import { DeleteTenantSection } from "@/features/tenants/components/delete-tenant-section";
import { getTenant } from "@/features/tenants/server";
import { DangerZone } from "@/shared/ui/danger-zone";
import { LeaveTenantSection } from "./leave-tenant-section";
import { TransferOwnershipSection } from "./transfer-ownership-section";

/** O bloco de sair ou transferir, que recebe o foco quando a posse muda. */
const MEMBERSHIP_ID = "membership";

/** O bloco da exclusão, que recebe o foco quando ela é agendada ou cancelada. */
const DELETION_ID = "deletion";

/**
 * A zona de perigo da aba Configurações, a última coisa dela: sair da
 * comunidade ou, para o owner, transferir a posse (com a exclusão agendada,
 * não) e excluir a comunidade. Na coluna de formulário (8 das 16 colunas a
 * partir do `lg`). Fica em `<Suspense>`.
 */
export async function CommunityDangerZone({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  const owner = tenant.role === "owner";

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <DangerZone>
          {!(owner && tenant.deletion_scheduled_for) && (
            <div id={MEMBERSHIP_ID} tabIndex={-1}>
              {owner ? (
                <TransferOwnershipSection slug={slug} focusId={MEMBERSHIP_ID} />
              ) : (
                <LeaveTenantSection slug={slug} />
              )}
            </div>
          )}
          {owner && (
            <div id={DELETION_ID} tabIndex={-1}>
              <DeleteTenantSection tenant={tenant} focusId={DELETION_ID} />
            </div>
          )}
        </DangerZone>
      </Column>
    </Grid>
  );
}
