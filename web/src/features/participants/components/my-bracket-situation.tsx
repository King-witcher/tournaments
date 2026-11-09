import { NextMatchLink } from "@/features/matches/components/next-match-link";
import { getMyTournamentEntry } from "@/features/tournaments/server";
import type { components } from "@/shared/api/schema";
import { BracketSituationTag } from "./bracket-situation-tag";
import { ParticipationTile } from "./participation-tile";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Sua situação" depois do início, para o inscrito: a etiqueta da situação
 * (`BracketSituationTag`) e a próxima partida como link. Lê "Seus torneios";
 * se a API falhar, cai no que o torneio diz (fez ou não o check-in). Quem
 * ainda joga vê o convite a ativar as notificações (`PushSuggestion`, abaixo
 * do bloco). Fica em `<Suspense>`.
 */
export async function MyBracketSituation({ tenant, tournament }: Props) {
  const entry = await getMyTournamentEntry(tenant, tournament.slug);
  return (
    <ParticipationTile
      suggestPush={
        (entry?.situation ??
          (tournament.check_in && !tournament.checked_in
            ? "missed_check_in"
            : "playing")) === "playing"
      }
      tag={<BracketSituationTag entry={entry} tournament={tournament} />}
    >
      {entry?.next_match && (
        <NextMatchLink
          next={entry.next_match}
          tournamentPath={`/${tenant}/${tournament.slug}`}
        />
      )}
    </ParticipationTile>
  );
}
