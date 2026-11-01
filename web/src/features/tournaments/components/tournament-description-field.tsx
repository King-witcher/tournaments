import { useTranslations } from "next-intl";
import { memo } from "react";

import { TextArea } from "@/shared/ui/text-area";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["description", "locale"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
};

/** A descrição no idioma original, em texto simples (as quebras de linha ficam). */
export const TournamentDescriptionField = memo(
  function TournamentDescriptionField({ values, onChange, errors }: Props) {
    const t = useTranslations("TournamentForm");

    return (
      <TextArea
        labelText={t("description")}
        name="description"
        rows={3}
        maxLength={2000}
        lang={values.locale}
        value={values.description}
        invalid={errors.description !== undefined}
        invalidText={errors.description}
        onChange={(e) => onChange({ description: e.target.value })}
      />
    );
  },
  sameValues(KEYS),
);
