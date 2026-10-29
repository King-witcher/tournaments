import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";

type Team = components["schemas"]["Team"];
type TeamMember = components["schemas"]["TeamMember"];

/** Os textos traduzidos de que o nome de um time precisa. */
export type TeamNameTexts = {
  /** `Member.deletedAccount`: no lugar do apelido de conta excluída. */
  deleted: string;
  /** `Team.teamOf`: "Time de {capitão}", para o time sem nome. */
  teamOf: (captain: string) => string;
};

/** O jogador de um time de um só (todo time do duelo); `null` nos outros. */
export function soloMember(team: Team): TeamMember | null {
  return team.members.length === 1 ? team.members[0] : null;
}

/** O capitão do time (sempre um dos jogadores). */
export function captainOf(team: Team): TeamMember | undefined {
  return (
    team.members.find((m) => m.user_id === team.captain) ?? team.members[0]
  );
}

/** Nenhum jogador do time continua na comunidade ("saiu", no duelo). */
export function teamLeft(team: Team): boolean {
  return team.members.every((m) => m.left);
}

/**
 * O nome exibido de um lado da partida, em toda tela: o nome do time; sem
 * nome e com um jogador (o duelo), o apelido dele, com "Conta excluída"
 * para a conta excluída; sem nome e com mais jogadores, "Time de {capitão}".
 */
export function teamName(team: Team, texts: TeamNameTexts): string {
  if (team.name) return team.name;
  const nickname = shownNickname(
    captainOf(team)?.nickname ?? "",
    texts.deleted,
  );
  return soloMember(team) ? nickname : texts.teamOf(nickname);
}
