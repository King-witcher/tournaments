import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  TEAM_SIZES,
  type TournamentValues,
  teamSizeOf,
} from "../lib/form-values";

type Schemas = components["schemas"];

const MODES: Schemas["TournamentMode"][] = ["duel", "tdm"];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
  /**
   * Na edição, o torneio salvo: com inscritos, trocar a modalidade ou o
   * tamanho avisa que os times serão refeitos ao salvar.
   */
  saved?: Pick<Schemas["Tournament"], "mode" | "team_size" | "participants">;
};

/**
 * A modalidade (duelo ou TDM, à vista) e, no TDM, o tamanho dos times. As
 * vagas não mudam sozinhas ao trocar: abaixo de dois times, a conferência
 * mostra o erro no campo delas.
 */
export function TournamentModeFields({
  values,
  onChange,
  errors,
  disabled,
  saved,
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
      <RadioGroup
        legendText={t("mode")}
        helperText={t(`modeHints.${values.mode}`)}
        name="mode"
        orientation="horizontal"
        value={values.mode}
        disabled={disabled}
        options={MODES.map((value) => ({ value, label: tMode(value) }))}
        // Do duelo (time de 1) para o TDM, o tamanho começa em 2; voltando
        // ao TDM antes de salvar, fica o que estava.
        onChange={(mode) =>
          onChange(
            mode === "tdm" && values.teamSize < 2
              ? { mode, teamSize: 2 }
              : { mode },
          )
        }
      />
      {values.mode === "tdm" && (
        <Select
          labelText={t("teamSize")}
          helperText={t("teamSizeHint", { size })}
          name="teamSize"
          value={values.teamSize}
          disabled={disabled}
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
      {reset && saved && (
        <Notice kind="warning">
          {t(`teamsReset.${reset}`, { count: saved.participants, size })}
        </Notice>
      )}
    </>
  );
}

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
