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
  /** Dentro de um item de acordeão, que já dá o título: sem a seção em volta. */
  bare?: boolean;
  /** Encerrado: as regras de cada idioma não mudam mais. */
  rulesLocked?: boolean;
};

/**
 * "Outros idiomas": um bloco por idioma diferente do original, cada um
 * começando sem tradução.
 */
export function TournamentTranslations({
  form,
  saved = [],
  bare = false,
  rulesLocked = false,
}: Props) {
  const t = useTranslations("TournamentForm");
  const blocks = form.others.map((l) => (
    <TournamentTranslation
      key={l}
      locale={l}
      original={form.original}
      draft={form.translationOf(l)}
      onChange={form.onTranslationChange}
      errors={form.translationErrors(l)}
      saved={saved.includes(l)}
      rulesLocked={rulesLocked}
    />
  ));

  if (bare) return blocks;
  return <FormSection legend={t("otherLanguages")}>{blocks}</FormSection>;
}
