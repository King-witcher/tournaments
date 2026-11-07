import { useTranslations } from "next-intl";
import { memo, useId, useState } from "react";

import type { components } from "@/shared/api/schema";
import { SelectItem } from "@/shared/ui/carbon/select";
import { ContainedList } from "@/shared/ui/contained-list";
import { ErrorText } from "@/shared/ui/error-text";
import { Select } from "@/shared/ui/select";
import { type FieldErrors, stageField } from "../lib/form-errors";
import type { StageValues } from "../lib/stage-values";
import {
  countingTiebreakers,
  counts,
  isTableFormat,
  moveTiebreaker,
  offTiebreakers,
  type Tiebreaker,
} from "../lib/tiebreakers";
import { type FocusOn, TiebreakerRow } from "./tiebreaker-row";
import styles from "./tournament-tiebreaker-fields.module.scss";

/** As opções do empate do fim, o padrão primeiro. */
const FINAL_TIES: components["schemas"]["FinalTie"][] = ["seed", "organizer"];

type Props = {
  stage: Pick<StageValues, "format" | "bestOf" | "tiebreakers" | "finalTie">;
  /** A posição da fase, a partir de 0. */
  index: number;
  /** Muda campos da fase `index`; a mesma função em todo render. */
  onStageChange: (index: number, patch: Partial<StageValues>) => void;
  errors: FieldErrors;
  /** A fase não muda: só os ligados, na ordem, sem as ações. */
  readOnly: boolean;
};

/**
 * Os desempates da fase de tabela, na ordem: os ligados numerados, com subir,
 * descer e desligar; embaixo, os desligados, com ligar (vai para o fim).
 * Setas em vez de arrastar, porque funcionam igual no teclado e no toque e a
 * lista tem no máximo cinco. O primeiro critério (saldo de séries ou pontos)
 * não aparece. Embaixo, quem decide o empate que sobra no fim: a ordem da
 * chave (o padrão) ou a organização, com a dica do que a escolhida faz. No
 * todos contra todos em md1 e md2, o saldo de partidas some, porque só
 * repetiria o saldo de séries, mas fica gravado se estava. Depois de cada
 * ação, o foco vai para o controle certo da linha, no lugar novo dela. Só
 * para leitura, os ligados, sem as ações; nenhum, "Só a ordem da chave".
 * Devolve dois blocos (a lista e o empate do fim), para quem chama pô-los
 * lado a lado.
 */
export const TournamentTiebreakerFields = memo(
  function TournamentTiebreakerFields({
    stage: values,
    index,
    onStageChange,
    errors,
    readOnly,
  }: Props) {
    const t = useTranslations("TournamentForm.tiebreakers");
    const errorId = useId();
    const [focus, setFocus] = useState<{
      tiebreaker: Tiebreaker;
      on: FocusOn;
      token: number;
    }>();
    const format = values.format;
    if (!isTableFormat(format)) return null;

    const list = values.tiebreakers;
    const shown = (x: Tiebreaker) => counts(x, format, values.bestOf);
    const ons = countingTiebreakers(list, format, values.bestOf);
    const offs = offTiebreakers(list, format, values.bestOf);
    const change = (
      next: Tiebreaker[],
      tiebreaker: Tiebreaker,
      on: FocusOn,
    ) => {
      onStageChange(index, { tiebreakers: next });
      setFocus((f) => ({ tiebreaker, on, token: (f?.token ?? 0) + 1 }));
    };
    const focusOf = (x: Tiebreaker) =>
      focus?.tiebreaker === x
        ? { focusToken: focus.token, focusOn: focus.on }
        : {};
    const error = errors[stageField(index, "tiebreakers")];
    const finalTie = (
      <Select
        labelText={t("finalTie")}
        helperText={t(`finalTieHints.${values.finalTie}`)}
        name={`stages.${index}.finalTie`}
        value={values.finalTie}
        readOnly={readOnly}
        onChange={(e) =>
          onStageChange(index, {
            finalTie: e.target.value as components["schemas"]["FinalTie"],
          })
        }
      >
        {FINAL_TIES.map((x) => (
          <SelectItem key={x} value={x} text={t(`finalTieOptions.${x}`)} />
        ))}
      </Select>
    );
    if (readOnly && ons.length === 0) {
      return (
        <>
          <p className={styles.hint}>{t("none")}</p>
          {finalTie}
        </>
      );
    }

    return (
      <>
        {/* O alvo do erro da API (`data-field`): a lista não tem campo nativo. */}
        <div
          className={styles.block}
          data-field={stageField(index, "tiebreakers")}
          tabIndex={-1}
          aria-describedby={error ? errorId : undefined}
        >
          <ContainedList label={t("label")} size="md">
            {ons.map((x, i) => (
              <TiebreakerRow
                key={x}
                tiebreaker={x}
                position={i + 1}
                readOnly={readOnly}
                onUp={
                  i > 0
                    ? () => change(moveTiebreaker(list, x, -1, shown), x, "up")
                    : undefined
                }
                onDown={
                  i < ons.length - 1
                    ? () => change(moveTiebreaker(list, x, 1, shown), x, "down")
                    : undefined
                }
                onOff={() =>
                  change(
                    list.filter((y) => y !== x),
                    x,
                    "on",
                  )
                }
                {...focusOf(x)}
              />
            ))}
            {(readOnly ? [] : offs).map((x) => (
              <TiebreakerRow
                key={x}
                tiebreaker={x}
                readOnly={false}
                onOn={() => change([...list, x], x, "up")}
                {...focusOf(x)}
              />
            ))}
          </ContainedList>
          {error && (
            <div id={errorId}>
              <ErrorText>{error}</ErrorText>
            </div>
          )}
        </div>
        {finalTie}
      </>
    );
  },
);
