"use client";

import { useTranslations } from "next-intl";
import type { ComponentProps, ReactNode } from "react";

import {
  ExpandableTile as CarbonExpandableTile,
  TileAboveTheFoldContent,
  TileBelowTheFoldContent,
} from "./carbon/tile";

type Props = Omit<
  ComponentProps<typeof CarbonExpandableTile>,
  "children" | "tileCollapsedIconText" | "tileExpandedIconText"
> & {
  /** O que aparece sempre (`TileAboveTheFoldContent`). */
  above: ReactNode;
  /** O que só aparece ao expandir (`TileBelowTheFoldContent`). */
  below: ReactNode;
};

/**
 * `ExpandableTile` do Carbon: um tile com o resumo à vista e o detalhe que
 * abre embaixo, para conteúdo secundário que ocuparia espaço demais. Sem
 * nada clicável dentro, o tile todo abre e fecha; com link ou botão, só a
 * seta (o Carbon troca sozinho). `tileCollapsedLabel` e `tileExpandedLabel`
 * põem um texto ao lado da seta ("Ver as regras"); `expanded` abre de
 * início. Vários blocos que abrem e fecham numa lista são o `Accordion`.
 * Traduz o rótulo da seta para o leitor de tela.
 */
export function ExpandableTile({ above, below, ...props }: Props) {
  const t = useTranslations("Common.expandableTile");
  return (
    <CarbonExpandableTile
      tileCollapsedIconText={t("expand")}
      tileExpandedIconText={t("collapse")}
      {...props}
    >
      <TileAboveTheFoldContent>{above}</TileAboveTheFoldContent>
      <TileBelowTheFoldContent>{below}</TileBelowTheFoldContent>
    </CarbonExpandableTile>
  );
}
