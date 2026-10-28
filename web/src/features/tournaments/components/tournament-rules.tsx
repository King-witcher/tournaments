import { getLocale, getTranslations } from "next-intl/server";

import { EmptyState } from "@/shared/ui/empty-state";
import { Markdown } from "@/shared/ui/markdown";
import { localizedTexts } from "../lib/localize";
import { loadTournament } from "../server";

type Props = { tenant: string; slug: string };

/**
 * A aba Regras: o texto das regras em Markdown, no idioma de quem vê (sem
 * tradução, o original). A aba só aparece com regras; quem abre o endereço
 * direto sem elas vê o aviso. Fica em `<Suspense>`.
 */
export async function TournamentRules({ tenant, slug }: Props) {
  const [t, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { rules } = localizedTexts(loaded.tournament, locale);
  return rules ? (
    <Markdown>{rules}</Markdown>
  ) : (
    <EmptyState>{t("noRules")}</EmptyState>
  );
}
