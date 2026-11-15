import type { components } from "@/shared/api/schema";
import { PROVIDERS, type Provider } from "./providers";

const DISMISSED_KEY = "backup-login-notice-dismissed";

/** Fechar o aviso da home vale 30 dias; depois ele volta. */
const DISMISSED_FOR_MS = 30 * 24 * 60 * 60 * 1000;

const CREATED_KEY = "account-created-by-provider";

/**
 * O provedor de quem depende só dele para entrar: sem email e com um único
 * provedor. Se ele ficar indisponível, a pessoa fica de fora. Com email, o
 * reserva já existe.
 */
export function soleProviderWithoutEmail(
  logins: components["schemas"]["LoginMethods"],
): Provider | undefined {
  if (logins.email) return undefined;
  const linked = PROVIDERS.filter((provider) => logins[provider]);
  return linked.length === 1 ? linked[0] : undefined;
}

/**
 * O aviso da home foi fechado neste navegador há menos de 30 dias. O valor
 * guardado é o momento do fechamento; o de antes, sem data (`"1"`), conta
 * como vencido.
 */
export function backupNoticeDismissed() {
  try {
    const elapsed = Date.now() - Number(localStorage.getItem(DISMISSED_KEY));
    return elapsed >= 0 && elapsed < DISMISSED_FOR_MS;
  } catch {
    return false;
  }
}

export function dismissBackupNotice() {
  try {
    localStorage.setItem(DISMISSED_KEY, String(Date.now()));
  } catch {
    // Sem o armazenamento, o aviso só volta na próxima visita.
  }
}

/**
 * O callback do provedor marca na aba que o login criou a conta agora, para a
 * página de destino (carregada do zero) avisar do login reserva.
 */
export function rememberAccountCreated() {
  try {
    sessionStorage.setItem(CREATED_KEY, "1");
  } catch {
    // Sem o armazenamento, fica o aviso da home.
  }
}

/** A marca do callback, que sai da aba ao ler: o aviso aparece uma vez. */
export function takeAccountCreated() {
  try {
    const created = sessionStorage.getItem(CREATED_KEY) !== null;
    sessionStorage.removeItem(CREATED_KEY);
    return created;
  } catch {
    return false;
  }
}
