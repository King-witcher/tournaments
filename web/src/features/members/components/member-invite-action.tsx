import { CommunityInviteButton } from "@/features/invites/components/community-invite-button";
import { getTenant } from "@/features/tenants/server";

/**
 * "Convidar", a ação primária de Membros: o link único da comunidade, só
 * para a organização e fora da exclusão agendada (ninguém mais entra). Fica
 * em `<Suspense>`.
 */
export async function MemberInviteAction({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  if (tenant.role === "player" || tenant.deletion_scheduled_for) return null;

  return <CommunityInviteButton slug={slug} name={tenant.name} />;
}
