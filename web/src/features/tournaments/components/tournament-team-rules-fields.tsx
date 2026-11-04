import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";
import { memo, useState } from "react";

import type { components } from "@/shared/api/schema";
import { Select } from "@/shared/ui/select";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { TEAM_PRESETS, type TeamPreset, teamPresetOf } from "../lib/team-rules";

type Schemas = components["schemas"];
type TeamRules = Schemas["TeamRules"];

const PRESETS: TeamPreset[] = ["captain", "everyone", "custom"];
const ACTORS: Schemas["TeamActor"][] = ["captain", "anyone"];
const PICKERS: Schemas["TeamPicker"][] = ["captain", "each_player"];

/** Os campos do formulário que o bloco lê. */
const KEYS = [
  "teamRules",
  "mapMode",
  "championPhase",
  "bansPerSeries",
] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/**
 * Quem age pelo time: o preset, com o que ele faz, e, em Personalizado, as
 * quatro regras. Só as regras vão para a API: o preset sai delas, e escolher
 * Personalizado continua nele até recarregar, mesmo com valores iguais aos de
 * um preset. O que não tem efeito (mapa desligado ou sorteado, campeões fora do
 * site, sem bans) não aparece, mas continua guardado.
 */
export const TournamentTeamRulesFields = memo(
  function TournamentTeamRulesFields({ values, onChange, disabled }: Props) {
    const t = useTranslations("TournamentForm.teamRules");
    const rules = values.teamRules;
    const [custom, setCustom] = useState(false);
    const preset = custom ? "custom" : teamPresetOf(rules);
    const update = (patch: Partial<TeamRules>) =>
      onChange({ teamRules: { ...rules, ...patch } });
    const actor = (field: "veto" | "ban" | "report") => (
      <Select
        labelText={t(`fields.${field}`)}
        name={`teamRules.${field}`}
        value={rules[field]}
        disabled={disabled}
        onChange={(e) =>
          update({ [field]: e.target.value as Schemas["TeamActor"] })
        }
      >
        {ACTORS.map((a) => (
          <SelectItem key={a} value={a} text={t(`actor.${a}`)} />
        ))}
      </Select>
    );

    return (
      <>
        <Select
          labelText={t("label")}
          helperText={t(`hints.${teamPresetOf(rules)}`)}
          name="teamPreset"
          value={preset}
          disabled={disabled}
          onChange={(e) => {
            const next = e.target.value as TeamPreset;
            setCustom(next === "custom");
            if (next !== "custom") onChange({ teamRules: TEAM_PRESETS[next] });
          }}
        >
          {PRESETS.map((p) => (
            <SelectItem key={p} value={p} text={t(`presets.${p}`)} />
          ))}
        </Select>
        {preset === "custom" && (
          <>
            {(values.mapMode === "veto" || values.mapMode === "vote") &&
              actor("veto")}
            {values.championPhase && values.bansPerSeries > 0 && actor("ban")}
            {values.championPhase && (
              <Select
                labelText={t("fields.pick")}
                name="teamRules.pick"
                value={rules.pick}
                disabled={disabled}
                onChange={(e) =>
                  update({ pick: e.target.value as Schemas["TeamPicker"] })
                }
              >
                {PICKERS.map((p) => (
                  <SelectItem key={p} value={p} text={t(`picker.${p}`)} />
                ))}
              </Select>
            )}
            {actor("report")}
          </>
        )}
      </>
    );
  },
  sameValues(KEYS),
);
