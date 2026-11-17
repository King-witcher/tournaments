import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LineChart } from "@/shared/ui/line-chart";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";

type Props = {
  history: components["schemas"]["RatingPoint"][];
};

/**
 * "Evolução do rating" na Visão geral da pessoa: o rating depois de cada
 * mapa que contou e de cada ajuste, no tempo (o histórico inteiro, que a API
 * manda em ordem). Sem nenhum, quem chama não mostra a seção.
 */
export function RatingHistoryChart({ history }: Props) {
  const t = useTranslations("Member.evolution");
  const group = t("value");
  return (
    <Panel>
      <Section title={t("title")}>
        <LineChart
          label={t("label")}
          valueTitle={group}
          points={history.map((p) => ({
            group,
            date: p.at,
            value: p.rating,
          }))}
        />
      </Section>
    </Panel>
  );
}
