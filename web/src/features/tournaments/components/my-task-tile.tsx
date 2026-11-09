import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useNextMatchLabel } from "@/features/matches/hooks/use-next-match-label";
import { CheckInButton } from "@/features/participants/components/check-in-button";
import { StageCheckInButton } from "@/features/stages/components/stage-check-in-button";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { localizedName } from "../lib/localize";
import { playerTask } from "../lib/my-tournaments";

type Schemas = components["schemas"];

type Props = {
  entry: Schemas["MyTournament"];
  /** O logo da comunidade, quando a lista leva. */
  avatar: ReactNode;
  /** Para onde o foco vai quando o check-in tira a tarefa da lista. */
  focusAfter: string;
};

/**
 * O que um torneio de quem vê espera dele: a série em que é a vez dele, com o
 * botão que leva a ela, ou o check-in que falta (do torneio ou de uma fase),
 * feito aqui mesmo (uma tarefa que se desfaz e não pede nada além do clique);
 * o da fase no duelo, sem o time na resposta, leva à página do torneio.
 */
export function MyTaskTile({ entry, avatar, focusAfter }: Props) {
  const t = useTranslations("Home.waiting");
  const tStages = useTranslations("Stages.checkIn");
  const locale = useLocale();
  const { tournament } = entry;
  const task = playerTask(entry);
  const teamName = useTeamName();

  if (task && entry.next_match) {
    return (
      <SeriesTaskTile
        entry={entry}
        next={entry.next_match}
        label={t(`action.${task}`)}
        avatar={avatar}
      />
    );
  }
  if (entry.situation === "stage_check_in_pending" && entry.stage) {
    // No duelo a API não manda o time (só no TDM): o botão leva à página do
    // torneio, onde o check-in da fase está à mão.
    return (
      <ActionTile
        avatar={avatar}
        title={localizedName(tournament, locale)}
        meta={`${entry.tenant_name} · ${t("stageCheckIn", { stage: entry.stage })}`}
        action={
          entry.team ? (
            <StageCheckInButton
              tenant={entry.tenant_slug}
              slug={tournament.slug}
              stage={entry.stage}
              team={entry.team.id}
              name={teamName(entry.team)}
              checkedIn={false}
              variant="tile"
              focusAfter={focusAfter}
            />
          ) : (
            <ButtonLink
              href={`/${entry.tenant_slug}/${tournament.slug}`}
              kind="tertiary"
              size="sm"
            >
              {tStages("do")}
            </ButtonLink>
          )
        }
      />
    );
  }
  return (
    <ActionTile
      avatar={avatar}
      title={localizedName(tournament, locale)}
      meta={entry.tenant_name}
      action={
        <CheckInButton
          tenant={entry.tenant_slug}
          slug={tournament.slug}
          kind="tertiary"
          size="sm"
          focusAfter={focusAfter}
        />
      }
    />
  );
}

function SeriesTaskTile({
  entry,
  next,
  label,
  avatar,
}: {
  entry: Schemas["MyTournament"];
  next: Schemas["NextMatch"];
  label: string;
  avatar: ReactNode;
}) {
  const tBracket = useTranslations("Bracket");
  const locale = useLocale();
  const nextLabel = useNextMatchLabel(next);
  const { tournament } = entry;

  return (
    <ActionTile
      avatar={avatar}
      title={localizedName(tournament, locale)}
      meta={
        <MetaLines
          lines={[
            `${entry.tenant_name} · ${nextLabel}`,
            next.next_map && tBracket("nextMap", { map: next.next_map.name }),
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${entry.tenant_slug}/${tournament.slug}/matches/${next.key}`}
          kind="tertiary"
          size="sm"
        >
          {label}
        </ButtonLink>
      }
    />
  );
}
