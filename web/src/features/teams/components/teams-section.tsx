import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import {
  hasStarted,
  participantsEditable,
  textsEditable,
} from "@/features/tournaments/lib/status";
import { loadTournament } from "@/features/tournaments/server";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { ButtonLink } from "@/shared/ui/button-link";
import { Notice } from "@/shared/ui/notice";
import { checkInShown } from "../lib/formation";
import { getFormation } from "../server";
import { RenameTeamButton } from "./rename-team-button";
import type { Candidate } from "./substitution-modal";
import { TeamActions } from "./team-actions";
import { TeamList } from "./team-list";

type Props = { tenant: string; slug: string };

/**
 * Os times e os reservas do TDM, só para ler, para todos: na aba Inscritos e
 * na página dos times depois de iniciar (ou para quem não organiza). O
 * capitão tem "Renomear" no próprio time; a organização, o ⋯ de cada time
 * (renomear, trocar o capitão e, em andamento, substituir e marcar a
 * desistência) e, antes de iniciar, o link "Formar times". Fica em
 * `<Suspense>`.
 */
export async function TeamsSection({ tenant, slug }: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const { status } = tournament;
  const organizer = me.role !== "player";
  const inProgress = status === "in_progress";
  const [t, formation, user, members] = await Promise.all([
    getTranslations("Team"),
    getFormation(tenant, slug),
    getCurrentUser(),
    organizer && inProgress ? activeMembers(tenant) : [],
  ]);
  if (!formation) {
    return <Notice kind="error">{await apiErrorMessage("internal")}</Notice>;
  }

  const editable = textsEditable(status);
  const playing = new Set([
    ...formation.teams.flatMap((team) => team.members.map((m) => m.user_id)),
    ...formation.reserves.map((p) => p.user_id),
  ]);
  const reserves: Candidate[] = formation.reserves
    .filter((p) => !p.left)
    .map((p) => ({ userId: p.user_id, nickname: p.nickname }));
  const others: Candidate[] = members
    .filter((m) => !playing.has(m.user_id))
    .map((m) => ({ userId: m.user_id, nickname: m.nickname }));

  const teamActions = (team: components["schemas"]["Team"]) => {
    if (organizer) {
      return (
        <TeamActions
          tenant={tenant}
          tournament={slug}
          team={team}
          editable={editable}
          inPlay={inProgress && team.seed != null && !team.withdrawn}
          reserves={reserves}
          others={others}
        />
      );
    }
    if (editable && user?.id === team.captain) {
      return <RenameTeamButton tenant={tenant} tournament={slug} team={team} />;
    }
    return undefined;
  };

  return (
    <TeamList
      tenant={tenant}
      formation={formation}
      checkIn={checkInShown(status, formation)}
      started={hasStarted(status)}
      teamActions={teamActions}
      teamsAction={
        organizer &&
        participantsEditable(status) && (
          <ButtonLink
            href={`/${tenant}/${slug}/teams`}
            kind="tertiary"
            size="sm"
          >
            {t("form")}
          </ButtonLink>
        )
      }
    />
  );
}

/** Os membros da comunidade, para quem entra numa substituição. */
async function activeMembers(tenant: string) {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/members", { params: { path: { slug: tenant } } })
    .then(({ data }) => data ?? [])
    .catch(() => []);
}
