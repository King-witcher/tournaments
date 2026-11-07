import { useLocale, useTranslations } from "next-intl";

import { useNextMatchLabel } from "@/features/matches/hooks/use-next-match-label";
import { TASK_STATUS, taskKey } from "@/features/matches/lib/status";
import { SITUATION_STATUS } from "@/features/participants/lib/situation";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { StatusTag } from "@/shared/ui/status-tag";
import { localizedName } from "../lib/localize";
import { TournamentMedia } from "./tournament-media";

type Schemas = components["schemas"];

/**
 * Um torneio de quem vê, como tile: com `media`, a capa no topo (ou o logo da
 * comunidade no lugar dela); o nome, a comunidade (no TDM, com o time) e o que
 * importa no momento: o início ou, em jogo, a próxima série. A etiqueta é o
 * que a série espera ou, sem isso, a situação dele. O tile todo leva ao torneio;
 * o caminho curto até a série está em "Esperando você".
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

  return (
    <LinkTile
      href={`/${entry.tenant_slug}/${tournament.slug}`}
      title={localizedName(tournament, locale)}
      media={
        media && (
          <TournamentMedia
            cover={tournament.cover_url}
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
            ]}
          />
        ) : (
          <>
            {entry.tenant_name}
            {" · "}
            {tournament.starts_at ? (
              <LocalDateTime value={tournament.starts_at} />
            ) : (
              t("noDate")
            )}
            {team}
          </>
        )
      }
      tags={
        entry.next_match?.task ? (
          <TaskTag next={entry.next_match} task={entry.next_match.task} />
        ) : (
          <StatusTag status={SITUATION_STATUS[entry.situation]}>
            {t(`situation.${entry.situation}`, {
              place: place ?? "",
              stage: entry.stage ?? "",
            })}
          </StatusTag>
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

function TaskTag({
  next,
  task,
}: {
  next: Schemas["NextMatch"];
  task: Schemas["ViewerTask"];
}) {
  const tBracket = useTranslations("Bracket");
  if (next.opponent_playing) {
    return (
      <StatusTag status="waiting">{tBracket("opponentPlaying")}</StatusTag>
    );
  }
  return (
    <StatusTag status={TASK_STATUS[task]}>
      {tBracket(`task.${taskKey(task, next.veto)}`)}
    </StatusTag>
  );
}
