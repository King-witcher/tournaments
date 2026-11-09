import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Accordion } from "@/shared/ui/accordion";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tile } from "@/shared/ui/carbon/tile";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { candidateState, entering, substitutes } from "../lib/candidates";
import { pendingStage } from "../lib/stages";
import { loadStage } from "../server";
import { StageCandidates } from "./stage-candidates";
import { StageOrganizerActions } from "./stage-organizer-actions";
import { StageStartDetails } from "./stage-start-details";
import { StageYourTeam } from "./stage-your-team";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner. */
  organizer: boolean;
};

/**
 * A fase seguinte à espera, na Visão geral,
 * entre o fim da fase anterior e o início dela: quem entra, na ordem, os
 * suplentes e, com check-in, quem confirmou. A organização abre o check-in e
 * inicia a fase (o Modal mostra quantos entram e quem fica de fora); o jogador de um time candidato
 * faz o check-in ou lê quando o suplente entra. No duelo, sem "time" no
 * texto. Sem fase assim, nada aparece.
 * Busca a fase, então fica em `<Suspense>`.
 */
export async function NextStage({ tenant, tournament, organizer }: Props) {
  const stage = pendingStage(tournament);
  if (!stage) return null;
  const [entry, user, t, tEntry] = await Promise.all([
    loadStage(tenant, tournament.slug, stage.number),
    getCurrentUser(),
    getTranslations("Stages"),
    getTranslations("Tournament.setup.entry"),
  ]);
  if (!entry) return null;

  const mine = user
    ? entry.candidates.find((candidate) =>
        candidate.team.members.some((member) => member.user_id === user.id),
      )
    : undefined;
  const tdm = tournament.mode === "tdm";
  const enters = entering(entry).length;
  const waiting = substitutes(entry).length;
  const active = entry.candidates.filter(
    (candidate) => candidateState(candidate, entry) !== "withdrawn",
  );
  const checkInOpen = entry.check_in && entry.status === "check_in";
  const blocker = entry.start_blocker
    ? await apiErrorMessage(entry.start_blocker, { min: 2, got: enters })
    : undefined;
  const rule = [
    entry.advance != null
      ? tEntry("top", { count: entry.advance, previous: entry.number - 1 })
      : tEntry("all", { previous: entry.number - 1 }),
    ...(entry.check_in ? [tEntry("checkIn")] : []),
  ].join(" ");
  const counts = [
    t("enters", { count: enters, tdm: tdm ? "yes" : "no" }),
    ...(waiting > 0 ? [t("substitutes", { count: waiting })] : []),
    ...(checkInOpen
      ? [
          t("confirmed", {
            count: active.filter((candidate) => candidate.checked_in).length,
            total: active.length,
          }),
        ]
      : []),
  ].join(" · ");

  return (
    <Tile>
      <Section
        title={t("title", { number: entry.number })}
        action={
          <StatusTag status={checkInOpen ? "check_in" : "waiting"}>
            {t(`state.${entry.status === "check_in" ? "check_in" : "pending"}`)}
          </StatusTag>
        }
      >
        <Stack gap={5}>
          <Stack gap={2}>
            <p>{rule}</p>
            <p>{counts}</p>
          </Stack>
          {mine && (
            <StageYourTeam
              tenant={tenant}
              slug={tournament.slug}
              entry={entry}
              candidate={mine}
              tdm={tdm}
              organizer={organizer}
            />
          )}
          {organizer && (
            <StageOrganizerActions
              tenant={tenant}
              slug={tournament.slug}
              stage={entry.number}
              openCheckIn={entry.check_in && entry.status === "pending"}
              blocker={blocker}
              details={<StageStartDetails entry={entry} tdm={tdm} />}
            />
          )}
          <Accordion title={t("list.title", { number: entry.number })}>
            <StageCandidates
              tenant={tenant}
              slug={tournament.slug}
              entry={entry}
              organizer={organizer}
              tdm={tdm}
            />
          </Accordion>
        </Stack>
      </Section>
    </Tile>
  );
}
