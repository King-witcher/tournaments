"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import type { GameValues } from "../lib/match-target";
import formStyles from "./game-form.module.scss";
import { MapSelect } from "./map-select";
import { ScoreFields } from "./score-fields";

type Schemas = components["schemas"];

type Props = {
  value: GameValues;
  onChange: (value: GameValues) => void;
  names: { a: string; b: string };
  /** Os mapas que cabem na escolha (sem os já jogados na série). */
  maps: Schemas["Map"][];
  /** O mapa que o veto deu: a proposta fica presa a ele, sem a escolha. */
  fixedMap?: Schemas["MapRef"];
  /** O mapa que vale agora, depois de conferir se a lista o oferece. */
  map: string;
};

/** O resultado que quem contesta diz ser o certo: o mapa e o placar. */
export function ContestProposalFields({
  value,
  onChange,
  names,
  maps,
  fixedMap,
  map,
}: Props) {
  const t = useTranslations("Match.form");
  return (
    <Stack gap={5}>
      {fixedMap ? (
        <p className={formStyles.fixedMap}>
          {t("vetoedMap", { map: fixedMap.name })}
        </p>
      ) : (
        <MapSelect
          maps={maps}
          value={map}
          onChange={(slug) => onChange({ ...value, map: slug })}
        />
      )}
      <ScoreFields
        names={names}
        a={value.a}
        b={value.b}
        onChange={(side, frags) => onChange({ ...value, [side]: frags })}
      />
    </Stack>
  );
}
