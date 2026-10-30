import { useTranslations } from "next-intl";
import { Suspense } from "react";

import {
  hasStarted,
  registrationOpen,
} from "@/features/tournaments/lib/status";
import type { components } from "@/shared/api/schema";
import { Countdown } from "@/shared/ui/countdown";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { SectionSkeleton } from "@/shared/ui/skeleton";
import { StatusTag } from "@/shared/ui/status-tag";
import { SITUATION_STATUS } from "../lib/situation";
import { MyBracketSituation } from "./my-bracket-situation";
import { ParticipationTile } from "./participation-tile";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner: sem inscrição, não precisa do aviso de por que não há botão. */
  organizer: boolean;
  /** Referência para saber se a abertura do check-in e o início já passaram. */
  now: Date;
};

/**
 * "Sua situação" para quem joga: a situação uma vez, na etiqueta, e só o que
 * a pessoa ainda precisa saber ou fazer (quando o check-in abre, o check-in
 * que falta, com o prazo e a contagem) e, depois de iniciar, a situação na
 * chave e a próxima partida. Para quem não está inscrito, só o motivo de não
 * haver botão de inscrição (só por link, lotado). Nada a
 * dizer, nada aparece. Para quem está inscrito e ainda vai jogar, a sugestão
 * de ativar as notificações num bloco à parte (`suggestPush`).
 */
export function MyParticipation({ tenant, tournament, organizer, now }: Props) {
  const t = useTranslations("Participants");
  const { status } = tournament;
  const tag = (situation: keyof typeof SITUATION_STATUS) => (
    <StatusTag status={SITUATION_STATUS[situation]}>
      {t(`situation.${situation}`, { place: "" })}
    </StatusTag>
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
      return <ParticipationTile tag={tag("checked_in")} suggestPush />;
    }
    const startsAt = tournament.starts_at;
    return (
      <ParticipationTile tag={tag("check_in_pending")} suggestPush>
        <p>{t("checkIn.open")}</p>
        {startsAt && new Date(startsAt) > now && (
          <p>
            {t.rich("checkIn.until", {
              date: () => <LocalDateTime value={startsAt} />,
              countdown: () => <Countdown until={startsAt} />,
            })}
          </p>
        )}
      </ParticipationTile>
    );
  }

  // Rascunho (inscrito pela organização) ou inscrição aberta.
  const opensAt = tournament.check_in_opens_at;
  const checkInHint =
    status === "registration" &&
    tournament.check_in_minutes != null &&
    opensAt != null &&
    new Date(opensAt) > now &&
    t.rich("checkIn.opensAt", {
      date: () => <LocalDateTime value={opensAt} />,
    });
  return (
    <ParticipationTile tag={tag("registered")} suggestPush>
      {checkInHint && <p>{checkInHint}</p>}
    </ParticipationTile>
  );
}
