import { Switcher } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./header-switcher.module.scss";

type Props = {
  /** Nome da lista, para o leitor de tela. */
  label: string;
  /** `SwitcherItem` e `SwitcherDivider` do Carbon. */
  children: ReactNode;
};

/**
 * Lista de links (ou botões) num painel do cabeçalho: o `Switcher` do
 * Carbon, com os itens crescendo para o texto longo (nome de comunidade)
 * quebrar a linha e com 48px no celular. Item sem estado de escolhido: o
 * Carbon não marca o item do painel, nem o da página atual (fica só o
 * `aria-current`).
 */
export function HeaderSwitcher({ label, children }: Props) {
  return (
    <Switcher aria-label={label} expanded className={styles.list}>
      {children}
    </Switcher>
  );
}
