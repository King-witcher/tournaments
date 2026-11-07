import { useTranslations } from "next-intl";
import { memo } from "react";

import type { components } from "@/shared/api/schema";
import { RadioGroup } from "@/shared/ui/radio-group";
import {
  poolOf,
  seriesLengths,
  type TournamentValues,
} from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { mapSequence, sequenceWords } from "../lib/veto-sequence";

type Schemas = components["schemas"];

// O padrão primeiro.
const ORDERS: Schemas["VetoOrder"][] = ["alternating", "bans_first"];

/** Os campos do formulário que o bloco lê. */
const KEYS = [
  "vetoOrder",
  "mapPool",
  "stageFormat",
  "bestOf",
  "finalsBestOf",
] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** Os mapas ativos (`GET /maps`), para contar o pool. */
  maps: Schemas["Map"][];
  /** O torneio já começou: só para leitura. */
  readOnly: boolean;
};

/**
 * A ordem do veto de mapas (alternado, o padrão, ou bans primeiro) e, embaixo, a
 * sequência que ela dá com o pool de agora, uma linha por melhor de N que o
 * torneio usa. Sem a linha de um melhor de N que o pool não comporta: o erro
 * do pool já aparece no campo dele.
 */
export const TournamentVetoOrderField = memo(function TournamentVetoOrderField({
  values,
  onChange,
  maps,
  readOnly,
}: Props) {
  const t = useTranslations("TournamentForm.veto");
  const pool = poolOf(values, maps).length;
  const label = ({ word, count }: ReturnType<typeof sequenceWords>[number]) =>
    count > 1
      ? t("run", { step: t(`steps.${word}`), count })
      : t(`steps.${word}`);
  const lines = seriesLengths(values).flatMap((bestOf) => {
    const steps = mapSequence(pool, bestOf, values.vetoOrder);
    return steps
      ? [
          <p key={bestOf}>
            {t("sequence", {
              bestOf,
              maps: pool,
              steps: sequenceWords(steps).map(label).join(", "),
            })}
          </p>,
        ]
      : [];
  });
  return (
    <RadioGroup
      legendText={t("order")}
      name="vetoOrder"
      orientation="horizontal"
      value={values.vetoOrder}
      options={ORDERS.map((value) => ({ value, label: t(`orders.${value}`) }))}
      onChange={(vetoOrder) => onChange({ vetoOrder })}
      helperText={lines.length > 0 ? lines : undefined}
      readOnly={readOnly}
    />
  );
}, sameValues(KEYS));
