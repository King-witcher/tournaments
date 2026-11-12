import { useTranslations } from "next-intl";

import { severity } from "@/features/organizing/lib/severity";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { TOURNAMENT_TITLE_ID } from "@/features/tournaments/lib/focus";
import type { components } from "@/shared/api/schema";
import { TaskNotice } from "@/shared/ui/task-notice";
import { StageCheckInButton } from "./stage-check-in-button";

/** O aviso do check-in da fase seguinte, para quem joga por um time candidato. */
export function StageCheckInNotice({
  tenant,
  slug,
  stage,
  candidate,
}: {
  tenant: string;
  slug: string;
  stage: number;
  candidate: components["schemas"]["StageCandidate"];
}) {
  const t = useTranslations("Home.tournaments");
  const teamName = useTeamName();
  return (
    <TaskNotice
      kind={severity("check_in")}
      title={t("situation.stage_check_in_pending", { stage })}
      action={
        <StageCheckInButton
          tenant={tenant}
          slug={slug}
          stage={stage}
          team={candidate.team.id}
          name={teamName(candidate.team)}
          checkedIn={false}
          variant="task"
          focusAfter={TOURNAMENT_TITLE_ID}
        />
      }
    />
  );
}
