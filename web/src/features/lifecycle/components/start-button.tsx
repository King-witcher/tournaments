import { getTranslations } from "next-intl/server";

import { getParticipants } from "@/features/participants/server";
import { checkInShown, formationCounts } from "@/features/teams/lib/formation";
import { getFormation } from "@/features/teams/server";
import type { components } from "@/shared/api/schema";
import type { ButtonKind } from "@/shared/ui/button";
import { TransitionButton } from "./transition-button";

/** Menor chave que a API aceita (`tournament.not_enough_eligible`). */
const MIN_ELIGIBLE = 2;

type Tournament = components["schemas"]["Tournament"];

type Props = {
  tenant: string;
  tournament: Tournament;
  kind?: ButtonKind;
};

/** O que o botão mostra: a pergunta e, se houver, o motivo. */
type Copy = { confirm: string; disabledReason?: string; note?: string };

/**
 * "Iniciar torneio" com a conta na confirmação. No duelo, no check-in,
 * quantos entram na chave e quantos ficam de fora sem check-in; com menos de
 * 2 elegíveis, o botão fica desabilitado, com o motivo. No TDM, quantos times
 * entram e quantos jogadores ficam de reserva; time incompleto (ou com
 * alguém sem check-in) é explicado sem travar o botão (a API responde
 * `tournament.teams_incomplete`).
 */
export async function StartButton({ tenant, tournament, kind }: Props) {
  const [t, copy] = await Promise.all([
    getTranslations("Lifecycle"),
    tournament.mode === "tdm"
      ? teamsCopy(tenant, tournament)
      : duelCopy(tenant, tournament),
  ]);
  return (
    <TransitionButton
      tenant={tenant}
      slug={tournament.slug}
      transition="start"
      label={t("start")}
      kind={kind}
      {...copy}
    />
  );
}

async function duelCopy(tenant: string, tournament: Tournament): Promise<Copy> {
  const t = await getTranslations("Lifecycle");
  const checkIn = tournament.status === "check_in";
  const list = checkIn
    ? await getParticipants(tenant, tournament.slug)
    : undefined;
  const eligible = list
    ? list.filter((p) => p.checked_in).length
    : tournament.participants;
  const confirm = !checkIn
    ? t("startConfirmNoCheckIn", { count: eligible })
    : list
      ? t("startConfirmCount", { in: eligible, out: list.length - eligible })
      : t("startConfirm");
  // Sem a lista (API falhou), não bloqueia: a API confere ao iniciar.
  const short = (!checkIn || list) && eligible < MIN_ELIGIBLE;
  return {
    confirm,
    disabledReason: short
      ? t(checkIn ? "notEnoughCheckIn" : "notEnough", {
          min: MIN_ELIGIBLE,
          count: eligible,
        })
      : undefined,
  };
}

async function teamsCopy(
  tenant: string,
  tournament: Tournament,
): Promise<Copy> {
  const t = await getTranslations("Lifecycle");
  const formation = await getFormation(tenant, tournament.slug);
  // Sem a formação (API falhou), não bloqueia: a API confere ao iniciar.
  if (!formation) return { confirm: t("startConfirm") };
  const checkIn = checkInShown(tournament.status, formation);
  const { complete, incomplete, reserves } = formationCounts(
    formation,
    checkIn,
  );
  const confirm = t("startConfirmTeams", { teams: complete, reserves });
  if (incomplete > 0) {
    return {
      confirm,
      note: t(checkIn ? "teamsIncompleteCheckIn" : "teamsIncomplete", {
        count: incomplete,
      }),
    };
  }
  return {
    confirm,
    disabledReason:
      complete < MIN_ELIGIBLE
        ? t("notEnoughTeams", { min: MIN_ELIGIBLE, count: complete })
        : undefined,
  };
}
