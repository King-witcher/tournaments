import { useTranslations } from "next-intl";
import { Suspense } from "react";
import { severity } from "@/features/organizing/lib/severity";
import { PushSuggestion } from "@/features/push/components/push-suggestion";
import {
  hasStarted,
  registrationOpen,
} from "@/features/tournaments/lib/status";
import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { StatusTag } from "@/shared/ui/status-tag";
import { TaskNotice } from "@/shared/ui/task-notice";
import { SITUATION_STATUS } from "../lib/situation";
import { CheckInButton } from "./check-in-button";
import { MyBracketSituation } from "./my-bracket-situation";
import { ParticipationTile } from "./participation-tile";
import { TeamSituationTag } from "./team-situation-tag";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner: sem inscrição, não precisa do aviso de por que não há botão. */
  organizer: boolean;
};

/**
 * "Sua situação" para quem joga: a situação uma vez, na etiqueta (no TDM, de
 * reserva se os times já se formaram sem a pessoa) e, depois de iniciar, a
 * situação na chave e a próxima partida. O check-in que falta é um aviso com
 * a ação, no lugar do bloco. Para quem não está inscrito, só o motivo de não
 * haver botão de inscrição (só por link, lotado). Nada a
 * dizer, nada aparece. Para quem está inscrito e ainda vai jogar, a sugestão
 * de ativar as notificações num bloco à parte (`suggestPush`).
 */
export function MyParticipation({ tenant, tournament, organizer }: Props) {
  const t = useTranslations("Participants");
  const { status } = tournament;
  const tag = (situation: keyof typeof SITUATION_STATUS) => (
    <StatusTag status={SITUATION_STATUS[situation]}>
      {t(`situation.${situation}`, { place: "", stage: "" })}
    </StatusTag>
  );
  // No TDM, inscrito ou com o check-in feito pode já estar de reserva (os
  // times foram formados sem ele): a etiqueta confere a formação.
  const teamTag = (situation: "registered" | "checked_in") =>
    tournament.mode === "tdm" ? (
      <Suspense fallback={tag(situation)}>
        <TeamSituationTag
          tenant={tenant}
          slug={tournament.slug}
          situation={situation}
        />
      </Suspense>
    ) : (
      tag(situation)
    );

  if (!tournament.registered) {
    if (organizer) return null;
    const text = !registrationOpen(status)
      ? null
      : tournament.admission === "invite_only"
        ? t("registration.inviteOnly")
        : tournament.participants >= tournament.max_participants
          ? t("registration.full")
          : null;
    return (
      text && (
        <ParticipationTile>
          <p>{text}</p>
        </ParticipationTile>
      )
    );
  }

  if (hasStarted(status)) {
    // A situação na chave e a próxima partida vêm de "Seus torneios".
    return (
      <Suspense fallback={<SectionSkeleton rows={1} />}>
        <MyBracketSituation tenant={tenant} tournament={tournament} />
      </Suspense>
    );
  }

  if (status === "check_in") {
    if (tournament.checked_in) {
      return <ParticipationTile tag={teamTag("checked_in")} suggestPush />;
    }
    return (
      <Stack gap={3}>
        <TaskNotice
          kind={severity("check_in")}
          title={t("checkIn.pendingTitle")}
          meta={t("checkIn.pendingBody")}
          action={
            <CheckInButton
              tenant={tenant}
              slug={tournament.slug}
              kind="ghost"
              size="sm"
            />
          }
        />
        <PushSuggestion />
      </Stack>
    );
  }

  return <ParticipationTile tag={teamTag("registered")} suggestPush />;
}
