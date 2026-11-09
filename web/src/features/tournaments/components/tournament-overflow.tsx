import { loadBracket } from "@/features/matches/server";
import { currentStage, startedStages } from "@/features/stages/lib/stages";
import type { components } from "@/shared/api/schema";
import { hasStarted, registrationOpen } from "../lib/status";
import { TournamentOverflowMenu } from "./tournament-overflow-menu";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner. */
  organizer: boolean;
  /** O owner, o único que exclui depois de começar. */
  owner: boolean;
};

/**
 * O ⋯ do cabeçalho, com o que é raro ou destrutivo e vale agora: desfazer o
 * próprio check-in, cancelar a própria inscrição e, para a organização, criar
 * outro torneio a partir deste (o Modal da criação, com ele em "Começar de"), fechar o check-in, voltar para rascunho, desfazer o início (com a conta dos resultados
 * que ele apaga, lida da chave; com várias fases, apaga todas) e, na fase 2 em
 * diante sem nada jogado, só o início dela e excluir (antes de começar, a organização;
 * depois, só o owner). Sem nada, não aparece.
 */
export async function TournamentOverflow({
  tenant,
  tournament,
  organizer,
  owner,
}: Props) {
  const { status, registered, checked_in: checkedIn } = tournament;
  const started = hasStarted(status);
  const flags = {
    undoCheckIn: status === "check_in" && registered && checkedIn,
    withdrawRegistration: registrationOpen(status) && registered,
    closeCheckIn: organizer && status === "check_in",
    canDelete: started ? owner : organizer,
  };
  let undoStart:
    | { results: number; hasCheckIn: boolean; several: boolean }
    | undefined;
  let undoStage: number | undefined;
  if (organizer && status === "in_progress") {
    // A mesma busca da aba da chave (cache por requisição): a fase atual.
    const loaded = await loadBracket(tenant, tournament.slug);
    const results = loaded.kind === "ok" ? loaded.data.results : 0;
    undoStart = {
      results,
      hasCheckIn: tournament.check_in,
      several: startedStages(tournament).length > 1,
    };
    // Desfazer só o início da fase 2 em diante, e só sem nada jogado nela.
    const current = currentStage(tournament);
    if (
      current.number > 1 &&
      current.status === "in_progress" &&
      results === 0
    ) {
      undoStage = current.number;
    }
  }
  if (!organizer && !undoStart && !Object.values(flags).some(Boolean)) {
    return null;
  }

  return (
    <TournamentOverflowMenu
      tenant={tenant}
      slug={tournament.slug}
      copySource={organizer ? tournament : undefined}
      unpublish={
        organizer && status === "registration"
          ? tournament.participants
          : undefined
      }
      undoStart={undoStart}
      undoStage={undoStage}
      inviteOnly={tournament.admission === "invite_only"}
      status={status}
      countsForRating={tournament.counts_for_rating}
      {...flags}
    />
  );
}
