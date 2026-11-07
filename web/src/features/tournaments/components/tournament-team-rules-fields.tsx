import { useTranslations } from "next-intl";
import { memo, useState } from "react";

import type { components } from "@/shared/api/schema";
import { SelectItem } from "@/shared/ui/carbon/select";
import { FormRow } from "@/shared/ui/form-row";
import { RadioTileGroup } from "@/shared/ui/radio-tile-group";
import { Select } from "@/shared/ui/select";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { TEAM_PRESETS, type TeamPreset, teamPresetOf } from "../lib/team-rules";

type Schemas = components["schemas"];
type TeamRules = Schemas["TeamRules"];

const PRESETS: TeamPreset[] = ["captain", "balanced", "everyone", "custom"];
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
  /** O torneio já começou: tudo aqui só para leitura. */
  readOnly: boolean;
};

/**
 * Quem age pelo time: os presets em tiles, numa fileira, cada um com o que
 * faz (quem marca "Pronto" fica de fora: quem joga vê na série), e, em
 * Personalizado, as quatro regras lado a lado, uma coluna por regra. Só as
 * regras vão para a API: o preset sai delas, e escolher
 * Personalizado continua nele até recarregar, mesmo com valores iguais aos de
 * um preset. O que não tem efeito (mapa desligado ou
 * sorteado, campeões fora do site, sem bans) não aparece, mas continua
 * guardado.
 */
export const TournamentTeamRulesFields = memo(
  function TournamentTeamRulesFields({ values, onChange, readOnly }: Props) {
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
        readOnly={readOnly}
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
        <RadioTileGroup
          legend={t("label")}
          name="teamPreset"
          value={preset}
          readOnly={readOnly}
          columns={4}
          options={PRESETS.map((p) => ({
            value: p,
            content: t(`presets.${p}`),
            description: t(`hints.${p}`),
          }))}
          onChange={(next) => {
            setCustom(next === "custom");
            if (next !== "custom") onChange({ teamRules: TEAM_PRESETS[next] });
          }}
        />
        {preset === "custom" && (
          <FormRow columns={4}>
            {(values.mapMode === "veto" || values.mapMode === "vote") &&
              actor("veto")}
            {values.championPhase && values.bansPerSeries > 0 && actor("ban")}
            {values.championPhase && (
              <Select
                labelText={t("fields.pick")}
                name="teamRules.pick"
                value={rules.pick}
                readOnly={readOnly}
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
          </FormRow>
        )}
      </>
    );
  },
  sameValues(KEYS),
);
