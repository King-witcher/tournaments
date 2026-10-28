import { useTranslations } from "next-intl";
import { Suspense } from "react";

import { hasStarted } from "@/features/tournaments/lib/status";
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
 * "Sua situação" para quem joga: inscrito (e quando o check-in abre),
 * check-in que falta (até o início, com a contagem) ou feito e, depois de
 * iniciar, a situação na chave e a próxima partida. Para quem não está
 * inscrito, só o motivo de não haver botão de inscrição (privado, lotado,
 * inscrições fechadas). Nada a dizer, nada aparece.
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
    const text =
      status === "registration"
        ? tournament.visibility === "private"
          ? t("registration.privateHint")
          : tournament.participants >= tournament.max_participants
            ? t("registration.full")
            : null
        : status === "check_in"
          ? t("checkIn.openNotRegistered")
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
      return (
        <ParticipationTile tag={tag("checked_in")}>
          <p>{t("checkIn.done")}</p>
        </ParticipationTile>
      );
    }
    const startsAt = tournament.starts_at;
    return (
      <ParticipationTile tag={tag("check_in_pending")}>
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
    // Com a abertura no passado e ainda em inscrição, a organização reabriu.
    (new Date(opensAt) <= now
      ? t("checkIn.suspended")
      : t.rich("checkIn.opensAt", {
          date: () => <LocalDateTime value={opensAt} />,
        }));
  return (
    <ParticipationTile tag={tag("registered")}>
      <p>{t("registration.registered")}</p>
      {checkInHint && <p>{checkInHint}</p>}
    </ParticipationTile>
  );
}
