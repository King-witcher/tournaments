import { useTranslations } from "next-intl";
import { memo } from "react";

import { Button } from "@/shared/ui/button";
import { SelectItem } from "@/shared/ui/carbon/select";
import { Checkbox } from "@/shared/ui/checkbox";
import { FormRow } from "@/shared/ui/form-row";
import { RadioTileGroup } from "@/shared/ui/radio-tile-group";
import { Select } from "@/shared/ui/select";
import { type FieldErrors, stageField } from "../lib/form-errors";
import { isTdm, type TournamentValues, teamSizeOf } from "../lib/form-values";
import { isEvenSeries, mapsToWin, teamsIn } from "../lib/format";
import { sameValues } from "../lib/same-values";
import type { StageValues } from "../lib/stage-values";
import { DEFAULT_TIEBREAKERS } from "../lib/tiebreakers";
import { BracketDiagram } from "./bracket-diagram";
import styles from "./tournament-format-fields.module.scss";

/**
 * A eliminação, o suíço e a disputa de posições: só ímpares, porque a série não
 * pode empatar.
 */
const BEST_OF = [1, 3, 5, 7, 9];

/** O todos contra todos também aceita as séries pares. */
const BEST_OF_ROUND_ROBIN = [1, 2, 3, 4, 5, 6, 7, 8, 9];

/** O teto de rodadas do suíço na API. */
const MAX_SWISS_ROUNDS = 127;

/**
 * A escolha do formato numa opção só: as duas eliminações, round robin, suíço
 * e disputa de posições.
 */
type FormatChoice = "double" | "single" | "round_robin" | "swiss" | "placement";

const CHOICES: FormatChoice[] = [
  "double",
  "single",
  "round_robin",
  "swiss",
  "placement",
];

const isElimination = (choice: FormatChoice) =>
  choice === "double" || choice === "single";

function choiceOf(
  stage: Pick<StageValues, "format" | "bracket">,
): FormatChoice {
  return stage.format === "elimination" ? stage.bracket : stage.format;
}

/**
 * Trocar de formato. Fora do todos contra todos, a série par vira a ímpar logo
 * acima (md4 vira md5), e o campo mostra o valor novo. Numa fase de tabela,
 * os desempates voltam ao padrão dela (o suíço começa pelo Buchholz).
 */
function patchOf(choice: FormatChoice, bestOf: number): Partial<StageValues> {
  if (choice === "round_robin")
    return { format: choice, tiebreakers: DEFAULT_TIEBREAKERS[choice] };
  const series = isEvenSeries(bestOf) ? { bestOf: bestOf + 1 } : {};
  if (choice === "placement") return { format: choice, ...series };
  return choice === "double" || choice === "single"
    ? { format: "elimination", bracket: choice, ...series }
    : {
        format: choice,
        tiebreakers: DEFAULT_TIEBREAKERS[choice],
        ...series,
      };
}

/** Os campos do torneio que o bloco lê (vagas e time contam as rodadas do suíço). */
const KEYS = ["mode", "teamSize", "maxParticipants"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  stage: StageValues;
  /** A posição da fase, a partir de 0. */
  index: number;
  /** É a última fase: só ela pode ser de eliminação. */
  last: boolean;
  /** Muda campos da fase `index`; a mesma função em todo render. */
  onStageChange: (index: number, patch: Partial<StageValues>) => void;
  errors: FieldErrors;
  /** A fase não muda: tudo aqui só para leitura. */
  readOnly: boolean;
  /**
   * Os mapas do pool salvo, quando o mapa sai dele: a série que pede mais
   * partidas que isso fica inválida na hora, sem esperar o erro da API.
   */
  pool?: number;
  /** Leva a Mapas, onde o pool cresce (com `pool`; a mesma função em todo render). */
  onGoToPool?: () => void;
};

/**
 * O formato da fase (as cinco opções em tiles, numa fileira, cada uma com o
 * que faz; a eliminação só na última fase), lado a lado as séries (e a final,
 * na eliminação; as pares só no todos contra todos, com a dica) e as rodadas
 * do suíço, e, na dupla, o reset embaixo da grande final, que é dela. O teto
 * de participantes do formato não aparece nos tiles: o campo Vagas o cobra
 * quando passa dele. No TDM, as rodadas contam times. A série que pede mais
 * mapas que o pool salvo fica inválida, com o caminho para Mapas embaixo.
 */
export const TournamentFormatFields = memo(function TournamentFormatFields({
  values,
  stage,
  index,
  last,
  onStageChange,
  errors,
  readOnly,
  pool,
  onGoToPool,
}: Props) {
  const t = useTranslations("TournamentForm");
  const tSettings = useTranslations("TournamentSettings");
  const choice = choiceOf(stage);
  const bestOfName = stageField(index, "bestOf");
  const roundsName = stageField(index, "swissRounds");
  // Cada partida da série precisa de um mapa do pool.
  const poolProblem = (games: number) =>
    pool !== undefined && games > pool
      ? t("seriesPool", { games, pool })
      : undefined;
  const bestOfProblem = poolProblem(stage.bestOf);
  const finalsProblem =
    stage.format === "elimination"
      ? poolProblem(stage.finalsBestOf)
      : undefined;
  const elimination = stage.format === "elimination";
  const items = (options: number[]) =>
    options.map((n) => (
      <SelectItem key={n} value={n} text={t("bestOfOption", { count: n })} />
    ));
  const roundRobin = stage.format === "round_robin";
  const tdm = isTdm(values);
  const size = teamSizeOf(values);
  // Rodadas que fazem sentido com as vagas: no máximo N − 1, com N os lados
  // da chave (no TDM, os times que as vagas comportam).
  const { teams } = teamsIn(values.maxParticipants ?? 2, size);
  const roundsMax = Math.max(1, Math.min(MAX_SWISS_ROUNDS, teams - 1));
  const roundOptions = Array.from({ length: roundsMax }, (_, i) => i + 1);
  if (stage.swissRounds !== null && stage.swissRounds > roundsMax) {
    roundOptions.push(stage.swissRounds);
  }
  const change = (patch: Partial<StageValues>) => onStageChange(index, patch);

  return (
    <>
      <RadioTileGroup
        legend={t("stageFormat")}
        name={stageField(index, "format")}
        value={choice}
        readOnly={readOnly}
        columns={5}
        options={CHOICES.map((value) => {
          const blocked = isElimination(value) && !last;
          return {
            value,
            content: t(`stageFormats.${value}`),
            description: blocked
              ? t("stages.eliminationLast")
              : // O teto de participantes do formato fica no campo Vagas, que
                // o cobra quando as vagas passam dele.
                t(`stageFormatHints.${value}`),
            media: <BracketDiagram format={value} />,
            disabled: blocked,
          };
        })}
        onChange={(next) => change(patchOf(next, stage.bestOf))}
      />
      <FormRow columns={4}>
        <Select
          labelText={t("bestOf")}
          helperText={
            isEvenSeries(stage.bestOf)
              ? t("bestOfEvenHint", { wins: mapsToWin(stage.bestOf) })
              : undefined
          }
          name={bestOfName}
          value={stage.bestOf}
          readOnly={readOnly}
          problem={bestOfProblem}
          invalid={errors[bestOfName] !== undefined}
          invalidText={errors[bestOfName]}
          onChange={(e) => change({ bestOf: Number(e.target.value) })}
        >
          {items(roundRobin ? BEST_OF_ROUND_ROBIN : BEST_OF)}
        </Select>
        {elimination && (
          <Select
            labelText={t(`finalsBestOf.${stage.bracket}`)}
            name={`stages.${index}.finalsBestOf`}
            value={stage.finalsBestOf}
            readOnly={readOnly}
            problem={finalsProblem}
            onChange={(e) => change({ finalsBestOf: Number(e.target.value) })}
          >
            {items(BEST_OF)}
          </Select>
        )}
        {stage.format === "swiss" && (
          <Select
            labelText={t("swissRounds")}
            helperText={t(tdm ? "swissRoundsHintTdm" : "swissRoundsHint")}
            name={roundsName}
            value={stage.swissRounds ?? "auto"}
            readOnly={readOnly}
            invalid={errors[roundsName] !== undefined}
            invalidText={errors[roundsName]}
            onChange={(e) =>
              change({
                swissRounds:
                  e.target.value === "auto" ? null : Number(e.target.value),
              })
            }
          >
            <SelectItem
              value="auto"
              text={t(tdm ? "swissRoundsAutoTdm" : "swissRoundsAuto")}
            />
            {roundOptions.map((n) => (
              <SelectItem
                key={n}
                value={n}
                text={t("swissRoundsOption", { count: n })}
              />
            ))}
          </Select>
        )}
        {(bestOfProblem || finalsProblem) && onGoToPool && (
          <div className={styles.goTo}>
            <Button kind="ghost" size="sm" onClick={onGoToPool}>
              {tSettings("goTo", { section: tSettings("sections.maps") })}
            </Button>
          </div>
        )}
      </FormRow>
      {elimination && stage.bracket === "double" && (
        <Checkbox
          labelText={t("grandFinalReset")}
          helperText={t("grandFinalResetHint")}
          name={`stages.${index}.grandFinalReset`}
          checked={stage.grandFinalReset}
          readOnly={readOnly}
          onChange={(grandFinalReset) => change({ grandFinalReset })}
        />
      )}
    </>
  );
}, sameValues(KEYS));
