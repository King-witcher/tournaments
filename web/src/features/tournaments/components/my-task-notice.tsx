import { useLocale, useTranslations } from "next-intl";

import { useMatchLabels } from "@/features/matches/hooks/use-match-labels";
import { useTaskNoticeText } from "@/features/matches/hooks/use-task-notice-text";
import type { Severity } from "@/features/organizing/lib/severity";
import { CheckInButton } from "@/features/participants/components/check-in-button";
import { StageCheckInButton } from "@/features/stages/components/stage-check-in-button";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import { localizedName } from "../lib/localize";
import { type PlayerTask, playerTask } from "../lib/my-tournaments";

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
 * O que um torneio de quem vê espera dele, com o torneio no metadado: a série
 * em que é a vez dele, com o botão que leva a ela, ou o check-in que falta
 * (do torneio ou de uma fase), feito aqui mesmo (uma tarefa que se desfaz e
 * não pede nada além do clique); o da fase no duelo, sem o time na resposta,
 * leva à página do torneio.
 */
export function MyTaskNotice({ kind, entry, community, focusAfter }: Props) {
  const t = useTranslations("Home");
  const locale = useLocale();
  const { tournament } = entry;
  const task = playerTask(entry);
  const teamName = useTeamName();
  const where = `${community ? `${community} · ` : ""}${localizedName(tournament, locale)}`;

  if (task && entry.next_match) {
    return (
      <SeriesTaskNotice
        kind={kind}
        where={where}
        entry={entry}
        next={entry.next_match}
        task={task}
      />
    );
  }
  if (entry.situation === "stage_check_in_pending" && entry.stage) {
    // No duelo a API não manda o time (só no TDM): o botão leva à página do
    // torneio, onde o check-in da fase está à mão.
    return (
      <TaskNotice
        kind={kind}
        title={t("tournaments.situation.stage_check_in_pending", {
          stage: entry.stage,
        })}
        meta={where}
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
              {t("waiting.openTournament")}
            </ButtonLink>
          )
        }
      />
    );
  }
  return (
    <TaskNotice
      kind={kind}
      title={t("tournaments.situation.check_in_pending")}
      meta={where}
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

/**
 * A série em que é a vez de quem vê: a situação no título e, embaixo do
 * torneio, a série e o adversário (com o mapa, ao enviar) ou o que fazer.
 */
function SeriesTaskNotice({
  kind,
  where,
  entry,
  next,
  task,
}: {
  kind: Severity;
  /** A comunidade (na lista de várias) e o torneio. */
  where: string;
  entry: Schemas["MyTournament"];
  next: Schemas["NextMatch"];
  task: PlayerTask;
}) {
  const t = useTranslations("Bracket.pending");
  const { title, note } = useTaskNoticeText();
  const { code } = useMatchLabels(next.single);
  const teamName = useTeamName();
  const opponent = next.opponent ? teamName(next.opponent) : undefined;
  const against = opponent && t("against", { match: code(next.key), opponent });
  const detail = next.task_detail;
  const line =
    task === "report"
      ? against &&
        `${against}${next.next_map ? ` · ${next.next_map.name}` : ""}`
      : task === "veto"
        ? against
        : note(task, detail);

  return (
    <TaskNotice
      kind={kind}
      title={title(task, detail, opponent)}
      meta={<MetaLines lines={[where, line]} />}
      action={
        <ButtonLink
          href={`/${entry.tenant_slug}/${entry.tournament.slug}/matches/${next.key}`}
          kind="ghost"
          size="sm"
        >
          {t("open")}
        </ButtonLink>
      }
    />
  );
}
