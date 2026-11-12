import { NextMatchLink } from "@/features/matches/components/next-match-link";
import { getMyTournamentEntry } from "@/features/tournaments/server";
import type { components } from "@/shared/api/schema";
import { BracketSituation } from "./bracket-situation";
import styles from "./my-situation-line.module.scss";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Sua situação" numa linha, no cabeçalho das seções do torneio depois do
 * início: a situação e a próxima série como link. Quem abre a Chave para
 * achar a série dele chega nela sem procurar o cartão; o bloco inteiro fica
 * na Visão geral. Fica em `<Suspense>`.
 */
export async function MySituationLine({ tenant, tournament }: Props) {
  const entry = await getMyTournamentEntry(tenant, tournament.slug);
  return (
    <div className={styles.line}>
      <BracketSituation entry={entry} tournament={tournament} />
      {entry?.next_match && (
        <NextMatchLink
          next={entry.next_match}
          tournamentPath={`/${tenant}/${tournament.slug}`}
        />
      )}
    </div>
  );
}
