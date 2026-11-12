import { useLocale, useTranslations } from "next-intl";

import { useNextMatchLabel } from "@/features/matches/hooks/use-next-match-label";
import { isActionable, taskKey } from "@/features/matches/lib/status";
import { SituationMark } from "@/features/participants/components/situation-mark";
import { hasSituationMark } from "@/features/participants/lib/situation";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { localizedName } from "../lib/localize";
import { TournamentMedia } from "./tournament-media";

type Schemas = components["schemas"];

/**
 * Um torneio de quem vê, como tile: com `media`, a capa no topo (ou o logo da
 * comunidade no lugar dela); o nome, a comunidade (no TDM, com o time) e o que
 * importa no momento: o início e a situação ou, em jogo, a próxima série e o
 * que ela espera de outro (a ação de quem vê já é um aviso no topo da página,
 * que leva à série). Etiqueta só na exceção da situação (fora da chave,
 * desistiu, reserva, empate a decidir). O tile todo leva ao torneio.
 */
export function MyTournamentTile({
  entry,
  media,
}: {
  entry: Schemas["MyTournament"];
  /** A grade tem alguma capa ou logo: este tile também leva o espaço. */
  media: boolean;
}) {
  const t = useTranslations("Home.tournaments");
  const tBracket = useTranslations("Bracket");
  const locale = useLocale();
  const teamName = useTeamName();
  const { tournament, placement } = entry;
  // No empate a decidir, a melhor posição do grupo ("Empate no 3º").
  const place = placement
    ? placement.tied > 1 && entry.situation !== "tie_pending"
      ? t("range", {
          from: placement.rank,
          to: placement.rank + placement.tied - 1,
        })
      : t("rank", { rank: placement.rank })
    : undefined;
  const team = entry.team && ` · ${t("team", { team: teamName(entry.team) })}`;
  const situation = t(`situation.${entry.situation}`, {
    place: place ?? "",
    stage: entry.stage ?? "",
  });
  const marked = hasSituationMark(entry.situation);
  const waiting = entry.next_match && waitingKey(entry.next_match);

  return (
    <LinkTile
      href={`/${entry.tenant_slug}/${tournament.slug}`}
      title={localizedName(tournament, locale)}
      media={
        media && (
          <TournamentMedia
            cover={tournament.cover_url}
            tenantCover={entry.tenant_cover_url}
            logo={entry.tenant_logo_url}
            community={entry.tenant_name}
          />
        )
      }
      meta={
        entry.next_match ? (
          <MetaLines
            lines={[
              `${entry.tenant_name}${team ?? ""}`,
              <NextMatchText key="next" next={entry.next_match} />,
              waiting && tBracket(waiting),
            ]}
          />
        ) : (
          <MetaLines
            lines={[
              <span key="when">
                {entry.tenant_name}
                {" · "}
                {tournament.starts_at ? (
                  <LocalDateTime value={tournament.starts_at} />
                ) : (
                  t("noDate")
                )}
                {team}
              </span>,
              !marked && situation,
            ]}
          />
        )
      }
      tags={
        marked && (
          <SituationMark situation={entry.situation} label={situation} />
        )
      }
    />
  );
}

function NextMatchText({ next }: { next: Schemas["NextMatch"] }) {
  const tBracket = useTranslations("Bracket");
  const label = useNextMatchLabel(next);
  return next.next_map
    ? `${label} · ${tBracket("nextMap", { map: next.next_map.name })}`
    : label;
}

/**
 * A chave (em `Bracket`) do que a série espera de outro: o adversário, a
 * organização, o capitão. A ação de quem vê fica de fora (`undefined`): é um
 * aviso no topo da página.
 */
function waitingKey(next: Schemas["NextMatch"]) {
  if (next.opponent_playing) return "opponentPlaying";
  if (!next.task || isActionable(next.task)) return undefined;
  return `task.${taskKey(next.task, next.veto)}` as const;
}
