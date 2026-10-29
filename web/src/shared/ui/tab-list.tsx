"use client";

import { TabList as CarbonTabList } from "@carbon/react";
import type { ComponentProps } from "react";

import styles from "./tab-list.module.scss";

type Props = ComponentProps<typeof CarbonTabList>;

/**
 * `TabList` do Carbon sem as setas de rolagem abaixo do `md`: lá a barra que
 * não cabe rola com o dedo. Com as setas, o Carbon mostra e esconde cada uma
 * conforme a rolagem, a largura da barra muda junto, e no celular as abas
 * ficavam tremendo ao trocar de aba (a seta some no fim, a barra alarga, a
 * rolagem se ajusta, a seta volta). Use no lugar do `TabList` do Carbon,
 * dentro do `Tabs`, com os mesmos `Tab`.
 */
export function TabList({ className, ...props }: Props) {
  // A classe vai na caixa em volta das setas: os `*OverflowButtonProps` do
  // Carbon trocariam a classe das setas, e elas perderiam o estilo e o
  // `--hidden` também no computador.
  return (
    <CarbonTabList
      className={className ? `${styles.tabs} ${className}` : styles.tabs}
      {...props}
    />
  );
}
