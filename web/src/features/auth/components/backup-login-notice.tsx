"use client";

import { useTranslations } from "next-intl";
import { useState, useSyncExternalStore } from "react";

import { useRouter } from "@/i18n/navigation";
import { ActionableNotification } from "@/shared/ui/actionable-notification";
import { LOGIN_SECTION } from "../lib/login-sections";
import type { Provider } from "../lib/providers";
import styles from "./backup-login-notice.module.scss";

const DISMISSED_KEY = "backup-login-notice-dismissed";

const noSubscription = () => () => {};

function readDismissed() {
  try {
    return localStorage.getItem(DISMISSED_KEY) !== null;
  } catch {
    return false;
  }
}

function rememberDismissed() {
  try {
    localStorage.setItem(DISMISSED_KEY, "1");
  } catch {
    // Sem o armazenamento, o aviso só volta na próxima visita.
  }
}

/**
 * Já na aba Logins, foca o campo; de fora, vai até ele. A página visitada antes
 * fica no DOM, escondida: achar o campo não basta, ele precisa estar à vista.
 */
function goToEmailField(router: ReturnType<typeof useRouter>) {
  const field = document.querySelector<HTMLInputElement>(
    `#${LOGIN_SECTION.email} input`,
  );
  if (field?.checkVisibility()) field.focus();
  else router.push(`/account/logins#${LOGIN_SECTION.email}`);
}

type Props = {
  /** O único jeito de entrar da conta. */
  provider: Provider;
  /** Dá para fechar, e fechar vale neste navegador (a home). */
  dismissible?: boolean;
};

/**
 * Para quem só entra por um provedor (`soleProviderWithoutEmail`): o risco e a
 * ação de pôr um email. É a `ActionableNotification` de baixo contraste do
 * Carbon, com o papel `status`, que não rouba o foco: o aviso espera a pessoa,
 * e abrir a home não pode prender o Tab nele. Na home, até hidratar não aparece,
 * para quem já fechou não ver o aviso piscar a cada visita.
 */
export function BackupLoginNotice({ provider, dismissible = false }: Props) {
  const t = useTranslations("Account");
  const router = useRouter();
  const stored = useSyncExternalStore(
    noSubscription,
    readDismissed,
    () => true,
  );
  const [closed, setClosed] = useState(false);
  if (dismissible && (stored || closed)) return null;

  return (
    <ActionableNotification
      className={styles.notice}
      kind="warning"
      role="status"
      inline
      lowContrast
      closeOnEscape={false}
      hideCloseButton={!dismissible}
      title={t("backup.title", { provider: t(`${provider}.title`) })}
      actionButtonLabel={t("backup.action")}
      onActionButtonClick={() => goToEmailField(router)}
      onCloseButtonClick={() => {
        rememberDismissed();
        setClosed(true);
      }}
    />
  );
}
