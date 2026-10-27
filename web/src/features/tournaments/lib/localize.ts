import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

/** Nome no idioma de quem vê; sem tradução, o original. */
export function localizedName(
  tournament: Pick<Schemas["TournamentSummary"], "name" | "translations">,
  locale: string,
) {
  return (
    tournament.translations.find((t) => t.locale === locale)?.name ??
    tournament.name
  );
}

/**
 * Nome, descrição e regras no idioma de quem vê. Descrição ou regras vazias
 * na tradução mostram as do original.
 */
export function localizedTexts(
  tournament: Schemas["Tournament"],
  locale: string,
) {
  const translation = tournament.translations.find((t) => t.locale === locale);
  return {
    name: translation?.name ?? tournament.name,
    description: translation?.description || tournament.description,
    rules: translation?.rules || tournament.rules,
  };
}
