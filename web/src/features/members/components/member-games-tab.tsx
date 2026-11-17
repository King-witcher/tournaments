import { getLocale, getTranslations } from "next-intl/server";

import { redirect } from "@/i18n/navigation";
import { apiErrorMessage } from "@/shared/api/errors";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { TableSkeleton } from "@/shared/ui/skeleton";
import { type GamesQuery, gamesHref } from "../lib/games-href";
import { getMember, getMemberGames } from "../server";
import { MemberGamesFilters } from "./member-games-filters";
import styles from "./member-games-tab.module.scss";
import { MemberGamesTable } from "./member-games-table";
import { PendingContent } from "./pending-content";
import { SearchTransition } from "./search-transition";

type Props = {
  tenant: string;
  /** O `user_id` do endereço, ainda como texto. */
  user: string;
  query: GamesQuery;
};

/**
 * A aba Partidas: todos os mapas confirmados da pessoa em todos os torneios
 * da comunidade, do mais recente, 25 por página (da API), com os filtros de
 * torneio, mapa e adversário em cima. Sem mapa nenhum, só o estado vazio
 * (sem filtros). Sem o membro, nada (a casca já mostra o erro). Fica em
 * `<Suspense>`.
 */
export async function MemberGamesTab({ tenant, user, query }: Props) {
  const member = await getMember(tenant, user);
  if (!member.ok) return null;
  const userId = member.data.user_id;
  const [loaded, t, tLoading] = await Promise.all([
    getMemberGames(tenant, userId, query),
    getTranslations("Member.games"),
    getTranslations("Member.loadingTab"),
  ]);
  if (!loaded.ok) {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  const page = loaded.data;
  const base = `/${tenant}/members/${userId}`;
  const filtered =
    query.tournament !== undefined ||
    query.map !== undefined ||
    query.opponent !== undefined;
  // A página além do fim (endereço antigo) vai para a última.
  if (page.games.length === 0 && page.total > 0) {
    const last = Math.ceil(page.total / page.page_size);
    redirect({
      href: gamesHref(base, { ...query, page: last }),
      locale: await getLocale(),
    });
  }

  return (
    <SearchTransition>
      <Stack gap={6}>
        <p className={styles.description}>{t("description")}</p>
        {page.filters.tournaments.length > 0 && (
          <MemberGamesFilters
            base={base}
            filters={page.filters}
            query={query}
          />
        )}
        <PendingContent
          label={tLoading("games")}
          fallback={<TableSkeleton columns={8} rows={10} />}
        >
          <MemberGamesTable
            tenant={tenant}
            base={base}
            page={page}
            query={query}
            filtered={filtered}
          />
        </PendingContent>
      </Stack>
    </SearchTransition>
  );
}
