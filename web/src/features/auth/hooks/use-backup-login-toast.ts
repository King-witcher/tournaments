import { useTranslations } from "next-intl";
import {
  useCallback,
  useEffect,
  useEffectEvent,
  useRef,
  useState,
} from "react";

import { usePathname } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { ToastItem } from "@/shared/ui/toast-stack";
import {
  backupNoticeDismissed,
  soleProviderWithoutEmail,
  takeAccountCreated,
} from "../lib/backup-login";
import { LOGIN_SECTION } from "../lib/login-sections";
import type { Provider } from "../lib/providers";

/** A chave do toast na pilha (as das notificações são números). */
export const BACKUP_LOGIN_TOAST = "backup-login";

const LOGINS_PATH = "/account/logins";

/** A página já mostra o aviso: os Logins da conta, sempre; a home, enquanto ele não foi fechado. */
function noticeOnPage(pathname: string) {
  return (
    pathname === LOGINS_PATH || (pathname === "/" && !backupNoticeDismissed())
  );
}

/**
 * O aviso de login reserva de quem acabou de criar a conta pelo Discord ou
 * Google, como item da pilha de toasts: uma vez, na página onde a pessoa chega
 * (a marca do callback sai da aba ao ler), até fechar. Confere os logins na
 * API, que dizem o provedor e garantem que a conta não tem outro jeito de
 * entrar. Não aparece, e sai da tela, na página que já mostra o aviso.
 * Independe do aviso da home: fechar um não fecha o outro.
 */
export function useBackupLoginToast() {
  const t = useTranslations("Account");
  const pathname = usePathname();
  const [provider, setProvider] = useState<Provider>();
  // No Strict Mode (dev), o efeito roda duas vezes; a marca só vale uma.
  const taken = useRef(false);

  const arrive = useEffectEvent(async () => {
    if (!takeAccountCreated() || noticeOnPage(pathname)) return;
    const logins = await callApi(api.GET("/auth/logins"));
    if (logins.ok) setProvider(soleProviderWithoutEmail(logins.data));
  });
  useEffect(() => {
    if (taken.current) return;
    taken.current = true;
    arrive();
  }, []);

  if (provider && noticeOnPage(pathname)) setProvider(undefined);

  const close = useCallback(() => setProvider(undefined), []);
  const item: ToastItem | undefined = provider && {
    id: BACKUP_LOGIN_TOAST,
    kind: "warning",
    sticky: true,
    title: t("backup.title", { provider: t(`${provider}.title`) }),
    link: {
      href: `${LOGINS_PATH}#${LOGIN_SECTION.email}`,
      label: t("backup.action"),
    },
  };
  return { item, close };
}
