import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

const SIDES = ["a", "b"] as const;

type Labels = {
  /** O nome exibido de cada lado (no duelo, o jogador). */
  names: { a: string; b: string };
  /** O nome de uma partida sem mapa ("Partida 2"). */
  game: (number: number) => string;
  /** O apelido como aparece na tela (`shownNickname`). */
  nickname: (nickname: string) => string;
};

/**
 * Os campeões de uma partida: o lado A, depois o B, cada um com o jogador. Na sala
 * sem nomes, os do lado juntos, com o nome do lado no fim:
 * `Athena Ranger(Time de king) Sorlag Keel(Time de thallis)`.
 */
function picksOf(
  game: Schemas["VetoGame"],
  labels: Labels,
  nameless: boolean,
): string[] {
  return SIDES.flatMap((side) => {
    const duel = game[`pick_${side}`];
    if (duel) return [`${duel.name}(${labels.names[side]})`];
    const picks = game.team_picks
      .filter((p) => p.side === side)
      .sort((p, q) => p.user_id - q.user_id);
    if (nameless) {
      const champions = picks.map((p) => p.champion.name).join(" ");
      return champions ? [`${champions}(${labels.names[side]})`] : [];
    }
    return picks.map(
      (p) => `${p.champion.name}(${labels.nickname(p.nickname)})`,
    );
  });
}

/**
 * O resultado do veto numa linha, para colar no chat:
 * `Tower of Koth - Athena(king) Ranger(thallis) | Crucible - Sorlag(king) Keel(thallis)`.
 * Sem draft, só os mapas. Sem mapa, o nome da partida no lugar dele. Partida
 * sem mapa nem campeão (votação ainda por vir) fica de fora.
 */
export function vetoResultText(
  veto: Schemas["VetoView"],
  labels: Labels,
): string {
  return veto.games
    .flatMap((game) => {
      const picks = picksOf(game, labels, veto.nameless);
      const title = game.map?.name;
      if (title)
        return [picks.length ? `${title} - ${picks.join(" ")}` : title];
      return picks.length
        ? [`${labels.game(game.game)} - ${picks.join(" ")}`]
        : [];
    })
    .join(" | ");
}
