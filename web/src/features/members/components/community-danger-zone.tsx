import { DeleteTenantSection } from "@/features/tenants/components/delete-tenant-section";
import { TENANT_TITLE_ID } from "@/features/tenants/lib/focus";
import { getTenant } from "@/features/tenants/server";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { DangerZone } from "@/shared/ui/danger-zone";
import { TransferOwnershipSection } from "./transfer-ownership-section";

/** O bloco da exclusão, que recebe o foco quando ela é agendada ou cancelada. */
const DELETION_ID = "deletion";

/**
 * A zona de perigo das Configurações, a última coisa delas, só do owner:
 * transferir a posse (com a exclusão agendada, não) e excluir a comunidade
 * (sair, quem não é owner faz no ⋯ do próprio perfil). Na coluna de
 * formulário (8 das 16 colunas a partir do `lg`). Fica em `<Suspense>`.
 */
export async function CommunityDangerZone({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  // Sair da comunidade fica no ⋯ do próprio perfil: a zona é só do owner.
  if (tenant.role !== "owner") return null;

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <DangerZone>
          {!tenant.deletion_scheduled_for && (
            // Depois de transferir, a zona some (quem vê virou organizer): o
            // foco vai para o título da comunidade.
            <TransferOwnershipSection slug={slug} focusId={TENANT_TITLE_ID} />
          )}
          <div id={DELETION_ID} tabIndex={-1}>
            <DeleteTenantSection tenant={tenant} focusId={DELETION_ID} />
          </div>
        </DangerZone>
      </Column>
    </Grid>
  );
}
