import { getTranslations } from "next-intl/server";

import { MemberName } from "@/features/members/components/member-name";
import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { ButtonLink } from "@/shared/ui/button-link";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDate } from "@/shared/ui/local-date";
import { Notice } from "@/shared/ui/notice";
import { getRanking } from "../server";
import { RatingValue } from "./rating-value";

/**
 * A aba Ranking: posição, membro (a linha abre a página da pessoa), rating
 * com a marca de provisório, mapas, vitórias e derrotas e o último mapa,
 * com busca. Fica em `<Suspense>`.
 */
export async function RankingTable({ slug }: { slug: string }) {
  const [loaded, ranking, t] = await Promise.all([
    getTenant(slug),
    getRanking(slug),
    getTranslations("Ratings"),
  ]);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  if (!ranking) {
    return <Notice kind="error">{await apiErrorMessage("internal")}</Notice>;
  }

  return (
    <DataTable
      label={t("ranking.label")}
      description={t("ranking.description", {
        initial: ranking.initial,
        count: ranking.provisional_games,
      })}
      primaryColumn="member"
      columns={[
        { key: "position", header: t("ranking.position") },
        { key: "member", header: t("ranking.member") },
        { key: "rating", header: t("ranking.rating") },
        { key: "games", header: t("ranking.games"), secondary: true },
        { key: "record", header: t("ranking.record"), secondary: true },
        { key: "lastGame", header: t("ranking.lastGame"), secondary: true },
      ]}
      rows={ranking.players.map((p) => ({
        id: String(p.user_id),
        href: `/${slug}/members/${p.user_id}`,
        search: `${p.nickname} ${p.display_name}`,
        cells: {
          position: t("member.positionShort", { position: p.position }),
          member: <MemberName member={p} />,
          rating: <RatingValue rating={p.rating} provisional={p.provisional} />,
          games: p.games,
          record: t("record", { wins: p.wins, losses: p.losses }),
          lastGame: p.last_game_at ? (
            <LocalDate value={p.last_game_at} />
          ) : (
            t("ranking.noGame")
          ),
        },
      }))}
      empty={{
        text: t("ranking.empty"),
        action: (
          <ButtonLink href={`/${slug}`} kind="tertiary">
            {t("ranking.browse")}
          </ButtonLink>
        ),
      }}
    />
  );
}
