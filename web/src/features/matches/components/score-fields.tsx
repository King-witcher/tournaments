"use client";

import { FormGroup } from "@carbon/react";
import { useTranslations } from "next-intl";

import { NumberInput } from "@/shared/ui/number-input";
import { isTied, MAX_FRAGS, MIN_FRAGS } from "../lib/score";
import styles from "./score-fields.module.scss";

type Props = {
  names: { a: string; b: string };
  a: number | null;
  b: number | null;
  onChange: (side: "a" | "b", value: number | null) => void;
};

/**
 * O placar da partida: os frags de cada lado, obrigatórios, lado a lado e
 * cada um embaixo do nome do seu lado (numa linha só, que corta o nome
 * longo). Quem fez mais vence, então o empate fica inválido na hora, no
 * segundo campo; quem usa decide o envio com `validScore`.
 */
export function ScoreFields({ names, a, b, onChange }: Props) {
  const t = useTranslations("Match.form");
  const tMatch = useTranslations("Match");
  const tied = isTied(a, b);

  const field = (side: "a" | "b", value: number | null) => (
    <div className={styles.field}>
      <NumberInput
        labelText={
          <span className={styles.name} title={names[side]}>
            {names[side]}
          </span>
        }
        value={value}
        onChange={(next) => onChange(side, next)}
        required
        allowEmpty
        hideSteppers
        disableWheel
        min={MIN_FRAGS}
        max={MAX_FRAGS}
        negative
        invalid={side === "b" && tied}
        invalidText={
          side === "b" && tied
            ? t("tied")
            : t("fragsRange", { min: MIN_FRAGS, max: MAX_FRAGS })
        }
      />
    </div>
  );

  return (
    <FormGroup legendText={t("score")}>
      <div className={styles.score}>
        {field("a", a)}
        <span aria-hidden className={styles.versus}>
          {tMatch("vs")}
        </span>
        {field("b", b)}
      </div>
    </FormGroup>
  );
}
