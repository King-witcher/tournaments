import { useTranslations } from "next-intl";
import { memo, useId, useState } from "react";

import { ContainedList } from "@/shared/ui/contained-list";
import { ErrorText } from "@/shared/ui/error-text";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
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

/** Os campos do formulário que o bloco lê. */
const KEYS = ["stageFormat", "bestOf", "tiebreakers"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: a ordem fica travada. */
  disabled: boolean;
};

/**
 * Os desempates da fase de tabela, na ordem: os ligados numerados, com subir,
 * descer e desligar; embaixo, os desligados, com ligar (vai para o fim).
 * Setas em vez de arrastar, porque funcionam igual no teclado e no toque e a
 * lista tem no máximo quatro. O primeiro critério (saldo de séries ou pontos)
 * não aparece, e a ordem da chave, sempre a última, vem no texto embaixo. No
 * todos contra todos em md1 e md2, o saldo de partidas some, porque só
 * repetiria o saldo de séries, mas fica gravado se estava. Depois de cada
 * ação, o foco vai para o controle certo da linha, no lugar novo dela.
 */
export const TournamentTiebreakerFields = memo(
  function TournamentTiebreakerFields({
    values,
    onChange,
    errors,
    disabled,
  }: Props) {
    const t = useTranslations("TournamentForm.tiebreakers");
    const errorId = useId();
    const [focus, setFocus] = useState<{
      tiebreaker: Tiebreaker;
      on: FocusOn;
      token: number;
    }>();
    const format = values.stageFormat;
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
      onChange({ tiebreakers: next });
      setFocus((f) => ({ tiebreaker, on, token: (f?.token ?? 0) + 1 }));
    };
    const focusOf = (x: Tiebreaker) =>
      focus?.tiebreaker === x
        ? { focusToken: focus.token, focusOn: focus.on }
        : {};
    const error = errors.tiebreakers;

    return (
      // O alvo do erro da API (`data-field`): a lista não tem campo nativo.
      <div
        className={styles.block}
        data-field="tiebreakers"
        tabIndex={-1}
        aria-describedby={error ? errorId : undefined}
      >
        <ContainedList label={t("label")} size="md">
          {ons.map((x, i) => (
            <TiebreakerRow
              key={x}
              tiebreaker={x}
              position={i + 1}
              disabled={disabled}
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
          {offs.map((x) => (
            <TiebreakerRow
              key={x}
              tiebreaker={x}
              disabled={disabled}
              onOn={() => change([...list, x], x, "up")}
              {...focusOf(x)}
            />
          ))}
        </ContainedList>
        <p className={styles.hint}>{t("seedLast")}</p>
        {error && (
          <div id={errorId}>
            <ErrorText>{error}</ErrorText>
          </div>
        )}
      </div>
    );
  },
  sameValues(KEYS),
);
