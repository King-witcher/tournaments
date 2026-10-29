import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { getSessions } from "../server";
import { SessionList } from "./session-list";

/**
 * A aba Sessões da conta: onde a conta está aberta. Fica em `<Suspense>`.
 */
export async function AccountSessions() {
  const result = await getSessions();
  if (!result) return null;
  if (!result.ok) {
    return <Notice kind="error">{await apiErrorMessage(result.code)}</Notice>;
  }
  // A atual primeiro; o resto na ordem da API (da mais nova para a mais antiga).
  const sessions = [...result.sessions].sort(
    (a, b) => Number(b.current) - Number(a.current),
  );
  return <SessionList sessions={sessions} />;
}
