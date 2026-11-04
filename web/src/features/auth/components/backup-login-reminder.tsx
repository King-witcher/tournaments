import { soleProviderWithoutEmail } from "../lib/backup-login";
import { getLogins } from "../server";
import { BackupLoginNotice } from "./backup-login-notice";

/**
 * Na home, o aviso de quem só entra por um provedor, que dá para fechar. Busca
 * os logins só aqui: `GET /me` fica sem campo novo, e uma ida a mais só nesta
 * página. Fica em `<Suspense>`.
 */
export async function BackupLoginReminder() {
  const result = await getLogins();
  if (!result?.ok) return null;
  const sole = soleProviderWithoutEmail(result.logins);
  return sole ? <BackupLoginNotice provider={sole} dismissible /> : null;
}
