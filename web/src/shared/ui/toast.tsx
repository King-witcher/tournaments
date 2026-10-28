"use client";

import { ToastNotification } from "@carbon/react";
import { useTranslations } from "next-intl";

import styles from "./toast.module.scss";

type Props = {
  open: boolean;
  onClose: () => void;
  /** O que aconteceu, curto ("Copiado!"). */
  title: string;
  kind?: "success" | "info";
  /** Some sozinho depois disto (ms). */
  timeout?: number;
};

/**
 * Confirmação passageira de algo que não muda a página ("copiado" e afins),
 * no canto de baixo. Para erro ou aviso que precisa ficar, use `Notice`.
 */
export function Toast({
  open,
  onClose,
  title,
  kind = "success",
  timeout = 3000,
}: Props) {
  const t = useTranslations("Common");
  if (!open) return null;
  return (
    <div className={styles.toast}>
      <ToastNotification
        kind={kind}
        role="status"
        lowContrast
        title={title}
        timeout={timeout}
        onClose={() => onClose()}
        aria-label={t("close")}
        statusIconDescription={t(`notice.${kind}`)}
      />
    </div>
  );
}
