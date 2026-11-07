import { useTranslations } from "next-intl";
import { memo } from "react";

import { Checkbox } from "@/shared/ui/checkbox";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["countsForRating"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** O torneio já começou: só para leitura. */
  readOnly: boolean;
};

/** Se cada partida atualiza o rating dos jogadores. */
export const TournamentRatingField = memo(function TournamentRatingField({
  values,
  onChange,
  readOnly,
}: Props) {
  const t = useTranslations("TournamentForm");

  return (
    <Checkbox
      labelText={t("countsForRating")}
      helperText={t("countsForRatingHint")}
      name="countsForRating"
      checked={values.countsForRating}
      readOnly={readOnly}
      onChange={(countsForRating) => onChange({ countsForRating })}
    />
  );
}, sameValues(KEYS));
