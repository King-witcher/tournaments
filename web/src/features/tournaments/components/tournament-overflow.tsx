import { loadBracket } from "@/features/matches/server";
import type { components } from "@/shared/api/schema";
import { registrationOpen } from "../lib/status";
import { TournamentOverflowMenu } from "./tournament-overflow-menu";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner. */
  organizer: boolean;
};

/**
 * O ⋯ do cabeçalho, com o que é raro ou destrutivo e vale agora: desfazer o
 * próprio check-in, cancelar a própria inscrição e, para a organização,
 * desfazer o início (com a conta dos resultados que ele
 * apaga, lida da chave) e excluir o rascunho. Sem nada, não aparece.
 */
export async function TournamentOverflow({
  tenant,
  tournament,
  organizer,
}: Props) {
  const { status, registered, checked_in: checkedIn } = tournament;
  const flags = {
    undoCheckIn: status === "check_in" && registered && checkedIn,
    withdrawRegistration: registrationOpen(status) && registered,
    deleteDraft: organizer && status === "draft",
  };
  let undoStart: { results: number; hasCheckIn: boolean } | undefined;
  if (organizer && status === "in_progress") {
    // A mesma busca da aba da chave (cache por requisição).
    const loaded = await loadBracket(tenant, tournament.slug);
    undoStart = {
      results: loaded.kind === "ok" ? loaded.data.results : 0,
      hasCheckIn: tournament.check_in_minutes != null,
    };
  }
  if (!undoStart && !Object.values(flags).some(Boolean)) return null;

  return (
    <TournamentOverflowMenu
      tenant={tenant}
      slug={tournament.slug}
      undoStart={undoStart}
      {...flags}
    />
  );
}
