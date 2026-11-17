"use client";

import "@carbon/charts-react/styles.css";

import type { LineChartOptions } from "@carbon/charts-react";
import { LineChart as CarbonLineChart } from "@carbon/charts-react";
import { enUS } from "date-fns/locale/en-US";
import { es } from "date-fns/locale/es";
import { ptBR } from "date-fns/locale/pt-BR";

import type { LineChartPoint } from "./line-chart";

type Props = {
  points: LineChartPoint[];
  options: LineChartOptions;
};

/** O idioma das marcas do eixo do tempo, que o Carbon Charts formata com o date-fns. */
const TIME_LOCALES = { pt: ptBR, es, en: enUS };

/**
 * O `LineChart` do Carbon Charts com o CSS dele: só o `LineChart` o importa,
 * por `next/dynamic`, para o pacote (d3 e date-fns incluídos) e o CSS só
 * chegarem à página que tem gráfico.
 */
export default function LineChartCanvas({ points, options }: Props) {
  const code = options.locale?.code as keyof typeof TIME_LOCALES | undefined;
  const localeObject = TIME_LOCALES[code ?? "pt"] ?? ptBR;
  return (
    <CarbonLineChart
      data={points}
      options={{ ...options, timeScale: { localeObject } }}
    />
  );
}
