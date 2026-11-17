"use client";

import type { LineChartOptions, ScaleTypes } from "@carbon/charts-react";
import dynamic from "next/dynamic";
import { useLocale, useTranslations } from "next-intl";
import { useSyncExternalStore } from "react";

import { useTheme } from "@/shared/hooks/use-theme";
import styles from "./line-chart.module.scss";

/** Um ponto da série: a data (ISO) e o valor. */
export type LineChartPoint = {
  /** O nome da série; com uma série só, o que o valor é ("Rating"). */
  group: string;
  date: string;
  value: number;
};

type Props = {
  points: LineChartPoint[];
  /** O que o gráfico mostra, para leitor de tela ("Rating ao longo do tempo"). */
  label: string;
  /** O título do eixo do valor ("Rating"). */
  valueTitle: string;
};

const DARK = "(prefers-color-scheme: dark)";

function subscribeDark(onChange: () => void) {
  const query = window.matchMedia(DARK);
  query.addEventListener("change", onChange);
  return () => query.removeEventListener("change", onChange);
}

/** O tema escuro está valendo: o escolhido ou, sem escolha, o do navegador. */
function useDark() {
  const { theme } = useTheme();
  const systemDark = useSyncExternalStore(
    subscribeDark,
    () => window.matchMedia(DARK).matches,
    () => false,
  );
  return theme === "dark" || (theme === "system" && systemDark);
}

/** O gráfico carregando, na altura dele (também para o esqueleto da página). */
export function LineChartSkeleton() {
  return <div aria-hidden className={styles.skeleton} />;
}

const Canvas = dynamic(() => import("./line-chart-canvas"), {
  ssr: false,
  loading: LineChartSkeleton,
});

/**
 * Gráfico de linha no tempo (`LineChart` do Carbon Charts), para como um
 * número mudou ao longo do tempo (a evolução do rating). Vai dentro de um
 * `Panel`: o tema do gráfico segue o do site (`white` no claro, `g90`, a
 * camada do `Panel`, no escuro) e muda junto com ele. O pacote do gráfico e o
 * CSS dele só carregam no navegador e só na página que o usa; enquanto
 * chegam, o esqueleto na altura do gráfico. Sem legenda (uma série só: o
 * título da seção diz o que é), sem a barra de ferramentas (a alternativa em
 * tabela é a lista que a página já tem) e com o tooltip do Carbon, com a data
 * e o valor no idioma da página. O eixo do valor não começa no zero: a
 * variação é pequena perto do valor.
 *
 * ```tsx
 * <LineChart points={[{ group: "Rating", date, value: 1540 }]} label="Rating ao longo do tempo" valueTitle="Rating" />
 * ```
 */
export function LineChart({ points, label, valueTitle }: Props) {
  const locale = useLocale();
  const dark = useDark();
  const t = useTranslations("Common.chart");
  const options: LineChartOptions = {
    theme: dark ? "g90" : "white",
    height: "100%",
    animations: false,
    legend: { enabled: false },
    toolbar: { enabled: false },
    accessibility: { svgAriaLabel: label },
    axes: {
      bottom: {
        mapsTo: "date",
        scaleType: "time" as ScaleTypes,
        title: t("date"),
      },
      left: {
        mapsTo: "value",
        scaleType: "linear" as ScaleTypes,
        title: valueTitle,
        includeZero: false,
      },
    },
    points: { radius: 3 },
    curve: "curveLinear",
    locale: {
      code: locale,
      number: (value, language) => value.toLocaleString(language),
      date: (value, language, format) =>
        value.toLocaleDateString(language, format),
      time: (value, language, format) =>
        value.toLocaleTimeString(language, format),
      translations: { group: t("group") },
    },
  };
  return (
    <div className={styles.chart}>
      <Canvas points={points} options={options} />
    </div>
  );
}
