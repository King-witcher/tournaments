import { useLocale, useTranslations } from "next-intl";

import { useNextMatchLabel } from "@/features/matches/hooks/use-next-match-label";
import type { Severity } from "@/features/organizing/lib/severity";
import { CheckInButton } from "@/features/participants/components/check-in-button";
import { StageCheckInButton } from "@/features/stages/components/stage-check-in-button";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import { localizedName } from "../lib/localize";
import { playerTask } from "../lib/my-tournaments";

type Schemas = components["schemas"];

type Props = {
  kind: Severity;
  entry: Schemas["MyTournament"];
  /** O nome da comunidade, quando a lista é de várias. */
  community?: string;
  /** Para onde o foco vai quando o check-in tira a tarefa da lista. */
  focusAfter: string;
};

/**
 * O que um torneio de quem vê espera dele: a série em que é a vez dele, com o
 * botão que leva a ela, ou o check-in que falta (do torneio ou de uma fase),
 * feito aqui mesmo (uma tarefa que se desfaz e não pede nada além do clique);
 * o da fase no duelo, sem o time na resposta, leva à página do torneio.
 */
export function MyTaskNotice({ kind, entry, community, focusAfter }: Props) {
  const t = useTranslations("Home.waiting");
  const tBracket = useTranslations("Bracket.pending.action");
  const tStages = useTranslations("Stages.checkIn");
  const locale = useLocale();
  const { tournament } = entry;
  const task = playerTask(entry);
  const teamName = useTeamName();
  const title = localizedName(tournament, locale);

  if (task && entry.next_match) {
    return (
      <SeriesTaskNotice
        kind={kind}
        title={title}
        entry={entry}
        next={entry.next_match}
        label={tBracket(task)}
        community={community}
      />
    );
  }
  if (entry.situation === "stage_check_in_pending" && entry.stage) {
    // No duelo a API não manda o time (só no TDM): o botão leva à página do
    // torneio, onde o check-in da fase está à mão.
    return (
      <TaskNotice
        kind={kind}
        title={title}
        meta={`${community ? `${community} · ` : ""}${t("stageCheckIn", { stage: entry.stage })}`}
        action={
          entry.team ? (
            <StageCheckInButton
              tenant={entry.tenant_slug}
              slug={tournament.slug}
              stage={entry.stage}
              team={entry.team.id}
              name={teamName(entry.team)}
              checkedIn={false}
              variant="task"
              focusAfter={focusAfter}
            />
          ) : (
            <ButtonLink
              href={`/${entry.tenant_slug}/${tournament.slug}`}
              kind="ghost"
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
    <TaskNotice
      kind={kind}
      title={title}
      meta={community}
      action={
        <CheckInButton
          tenant={entry.tenant_slug}
          slug={tournament.slug}
          kind="ghost"
          size="sm"
          focusAfter={focusAfter}
        />
      }
    />
  );
}

function SeriesTaskNotice({
  kind,
  title,
  entry,
  next,
  label,
  community,
}: {
  kind: Severity;
  title: string;
  entry: Schemas["MyTournament"];
  next: Schemas["NextMatch"];
  label: string;
  community?: string;
}) {
  const tBracket = useTranslations("Bracket");
  const nextLabel = useNextMatchLabel(next);

  return (
    <TaskNotice
      kind={kind}
      title={title}
      meta={
        <MetaLines
          lines={[
            `${community ? `${community} · ` : ""}${nextLabel}`,
            next.next_map && tBracket("nextMap", { map: next.next_map.name }),
          ]}
        />
      }
      action={
        <ButtonLink
          href={`/${entry.tenant_slug}/${entry.tournament.slug}/matches/${next.key}`}
          kind="ghost"
          size="sm"
        >
          {label}
        </ButtonLink>
      }
    />
  );
}
