import { getLocale, getTranslations } from "next-intl/server";

import { EmptyState } from "@/shared/ui/empty-state";
import { UserText } from "@/shared/ui/user-text";
import { rulesSource } from "../lib/localize";
import { textsEditable } from "../lib/status";
import { loadTournament } from "../server";
import { RulesEditor } from "./rules-editor";

type Props = { tenant: string; slug: string };

/**
 * A aba Regras: o texto das regras (texto simples, com as quebras de linha),
 * no idioma de quem vê (sem tradução, o original). Para a organização, até o
 * torneio encerrar, "Editar regras" ali mesmo, e a aba aparece mesmo sem
 * regras; para os outros, só com regras (quem abre o endereço direto sem
 * elas vê o aviso). Fica em `<Suspense>`.
 */
export async function TournamentRules({ tenant, slug }: Props) {
  const [t, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const source = rulesSource(tournament, locale);
  if (me.role !== "player" && textsEditable(tournament.status)) {
    return (
      <RulesEditor
        tenant={tenant}
        tournament={tournament.slug}
        locale={source.locale}
        rules={source.rules}
        translation={source.translation}
      />
    );
  }
  return source.rules ? (
    <UserText>{source.rules}</UserText>
  ) : (
    <EmptyState>{t("noRules")}</EmptyState>
  );
}
