import { getCurrentUser } from "@/features/auth/server";
import { NotificationBellPanel } from "./notification-bell-panel";

/**
 * O sininho no cabeçalho, só com sessão. Um por conta (`key`): a lista
 * guardada da última abertura não passa para outra conta. Lê a sessão (a
 * mesma busca do menu da pessoa): fica em `<Suspense>`.
 */
export async function NotificationBell() {
  const user = await getCurrentUser();
  if (!user) return null;
  return <NotificationBellPanel key={user.id} />;
}
