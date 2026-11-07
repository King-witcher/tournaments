import { useTranslations } from "next-intl";
import { memo, type ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { SelectItem } from "@/shared/ui/carbon/select";
import { FormRow } from "@/shared/ui/form-row";
import { Notice } from "@/shared/ui/notice";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  TEAM_SIZES,
  type TournamentValues,
  teamSizeOf,
} from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { TEAM_PRESETS } from "../lib/team-rules";

type Schemas = components["schemas"];

const MODES: Schemas["TournamentMode"][] = ["duel", "tdm"];

/** Os campos do formulário que o bloco lê. */
const KEYS = ["mode", "teamSize"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui só para leitura. */
  readOnly: boolean;
  /**
   * Na edição, o torneio salvo: com inscritos, trocar a modalidade ou o
   * tamanho avisa que os times serão refeitos ao salvar.
   */
  saved?: Pick<Schemas["Tournament"], "mode" | "team_size" | "participants">;
  /**
   * Campos curtos na mesma linha, depois do tamanho (as vagas, no Formato).
   * Com eles, a linha vai até quatro colunas, as mesmas das séries embaixo;
   * sem eles (o Modal de criação), duas.
   */
  children?: ReactNode;
};

/**
 * A modalidade (duelo ou TDM, à vista) e, no TDM, o tamanho dos times ao lado
 * dela (onde cabe; embaixo, no celular), seguidos dos `children` na mesma
 * linha. As vagas não mudam sozinhas ao trocar: abaixo de dois times, a
 * conferência mostra o erro no campo delas.
 */
export const TournamentModeFields = memo(function TournamentModeFields({
  values,
  onChange,
  errors,
  readOnly,
  saved,
  children,
}: Props) {
  const t = useTranslations("TournamentForm");
  const tMode = useTranslations("Tournaments.mode");
  const size = teamSizeOf(values);
  const reset =
    saved && saved.participants > 0
      ? teamsReset(saved, values.mode, size)
      : undefined;

  return (
    <>
      <FormRow columns={children ? 4 : 2}>
        <RadioGroup
          legendText={t("mode")}
          name="mode"
          orientation="horizontal"
          value={values.mode}
          readOnly={readOnly}
          options={MODES.map((value) => ({ value, label: tMode(value) }))}
          // Do duelo (time de 1) para o TDM, o tamanho começa em 2 e quem age
          // pelo time em Balanceado, como todo time novo; voltando ao TDM antes
          // de salvar, fica o que estava.
          onChange={(mode) =>
            onChange(
              mode === "tdm" && values.teamSize < 2
                ? { mode, teamSize: 2, teamRules: TEAM_PRESETS.balanced }
                : { mode },
            )
          }
        />
        {values.mode === "tdm" && (
          <Select
            labelText={t("teamSize")}
            name="teamSize"
            value={values.teamSize}
            readOnly={readOnly}
            invalid={errors.teamSize !== undefined}
            invalidText={errors.teamSize}
            onChange={(e) => onChange({ teamSize: Number(e.target.value) })}
          >
            {TEAM_SIZES.map((n) => (
              <SelectItem
                key={n}
                value={n}
                text={t("teamSizeOption", { size: n })}
              />
            ))}
          </Select>
        )}
        {children}
      </FormRow>
      {reset && saved && (
        <Notice kind="warning">
          {t(`teamsReset.${reset}`, { count: saved.participants, size })}
        </Notice>
      )}
    </>
  );
}, sameValues(KEYS));

/**
 * O que salvar faz com os times dos inscritos, se a modalidade ou o tamanho
 * mudou: do duelo para o TDM, todos ficam sem time; do TDM para o duelo,
 * cada um volta a jogar sozinho; outro tamanho, os times são desfeitos.
 */
function teamsReset(
  saved: Pick<Schemas["Tournament"], "mode" | "team_size">,
  mode: Schemas["TournamentMode"],
  size: number,
) {
  if (mode === saved.mode && size === saved.team_size) return undefined;
  if (saved.mode === "duel") return "fromDuel";
  return mode === "duel" ? "toDuel" : "resize";
}
