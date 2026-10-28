import { getCurrentUser } from "@/features/auth/server";
import { TenantSwitcherPanel } from "./tenant-switcher-panel";

/**
 * Suas comunidades no cabeçalho, só com sessão. Uma por conta (`key`): a
 * lista guardada da última abertura não passa para outra conta. Lê a sessão
 * (a mesma busca do menu da pessoa): fica em `<Suspense>`, à esquerda das
 * outras ações, que não se mexem se ela não aparecer (a barra é alinhada à
 * direita).
 */
export async function TenantSwitcher() {
  const user = await getCurrentUser();
  if (!user) return null;
  return <TenantSwitcherPanel key={user.id} />;
}
