import { getCurrentUser } from "@/features/auth/server";
import { PushLocaleSync } from "./push-locale-sync";

/**
 * Para quem está logado: a inscrição de Web Push deste navegador acompanha o
 * idioma e a conta. Lê a sessão: fica em `<Suspense>`, sem esqueleto (não
 * desenha nada).
 */
export async function PushSession() {
  const user = await getCurrentUser();
  return user ? <PushLocaleSync userId={user.id} /> : null;
}
