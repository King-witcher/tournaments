"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import styles from "./required-label.module.scss";

/**
 * O rótulo de um campo, com a estrela vermelha quando ele é obrigatório. A
 * estrela some para o leitor de tela, que lê "obrigatório" no lugar. Os
 * campos de `shared/ui` já o usam com `required`; use direto só em campo
 * montado à mão.
 */
export function RequiredLabel({
  required,
  children,
}: {
  required?: boolean;
  children: ReactNode;
}) {
  const t = useTranslations("Common.validation");
  if (!required) return children;
  return (
    <>
      {children}
      <span className={styles.mark} aria-hidden="true">
        *
      </span>
      <span className={styles.hidden}> {t("requiredHidden")}</span>
    </>
  );
}
