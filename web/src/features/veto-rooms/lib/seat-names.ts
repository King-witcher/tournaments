import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Seat = Schemas["VetoRoomSeat"];

/** Os nomes de quem não entrou: traduzidos, a API só diz que está vazio. */
export type SeatLabels = {
  /** "Lado A". */
  side: (side: Schemas["Side"]) => string;
  /** "Time de king". */
  team: (name: string) => string;
  /** "Lugar A2". */
  seat: (seat: Seat) => string;
  /** No histórico, que não diz qual é o lugar. */
  empty: string;
};

/**
 * O nome exibido de cada lado: no duelo, o jogador; nos times, o time do
 * primeiro que entrou; sem ninguém, o lado.
 */
export function sideNames(
  room: { team_size: number; seats: Seat[] },
  labels: SeatLabels,
) {
  const name = (side: Schemas["Side"]) => {
    const first = room.seats.find((s) => s.side === side && s.name)?.name;
    if (!first) return labels.side(side);
    return room.team_size === 1 ? first : labels.team(first);
  };
  return { a: name("a"), b: name("b") };
}

/**
 * O veto com o lugar vazio escrito: a API manda o nome vazio, e os tabuleiros
 * são os da série, em que vazio quer dizer conta excluída. Na sala sem nomes
 * (`nameless`), o lugar sem link fica sem nome, e a tela mostra só os campeões.
 */
export function withSeatNames(
  veto: Schemas["VetoView"],
  seats: Seat[],
  labels: SeatLabels,
): Schemas["VetoView"] {
  const named = <T extends { user_id: number; nickname: string }>(
    player: T,
  ) => {
    if (player.nickname !== "") return player;
    const seat = seats.find((s) => s.id === player.user_id);
    return { ...player, nickname: seat ? labels.seat(seat) : labels.empty };
  };
  if (veto.nameless) return veto;
  return {
    ...veto,
    ready_waiting: veto.ready_waiting.map(named),
    missing: veto.missing.map(named),
    games: veto.games.map((g) => ({
      ...g,
      team_picks: g.team_picks.map(named),
      hidden_picks: g.hidden_picks.map(named),
    })),
    history: veto.history.map((e) =>
      e.player === "" ? { ...e, player: labels.empty } : e,
    ),
  };
}
