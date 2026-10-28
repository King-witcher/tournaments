import { getCurrentUser } from "@/features/auth/server";
import { LiveConnection } from "./live-connection";

/**
 * O que só existe para quem está logado: a conexão com `/ws`. Uma por conta
 * (`key`): trocar de conta no mesmo navegador reabre a conexão. Lê a sessão:
 * fica em `<Suspense>`, sem esqueleto (não desenha nada).
 */
export async function RealtimeSession() {
  const user = await getCurrentUser();
  if (!user) return null;
  return <LiveConnection key={user.id} />;
}
