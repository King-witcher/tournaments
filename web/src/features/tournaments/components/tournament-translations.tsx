import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { FormSection } from "@/shared/ui/form-section";
import type { TournamentFormState } from "../hooks/use-tournament-form";
import { TournamentTranslation } from "./tournament-translation";

type Locale = components["schemas"]["Locale"];

type Props = {
  form: TournamentFormState;
  /** Os idiomas que o torneio já tem traduzidos (na edição). */
  saved?: Locale[];
};

/**
 * "Outros idiomas": um bloco por idioma diferente do original, cada um
 * começando sem tradução.
 */
export function TournamentTranslations({ form, saved = [] }: Props) {
  const t = useTranslations("TournamentForm");

  return (
    <FormSection legend={t("otherLanguages")}>
      {form.others.map((l) => (
        <TournamentTranslation
          key={l}
          locale={l}
          original={form.original}
          draft={form.translationOf(l)}
          onChange={(patch) => form.onTranslationChange(l, patch)}
          errors={form.translationErrors(l)}
          saved={saved.includes(l)}
        />
      ))}
    </FormSection>
  );
}
