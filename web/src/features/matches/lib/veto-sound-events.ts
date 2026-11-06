import type { components } from "@/shared/api/schema";
import { isActing, isWindow } from "./veto";

type Schemas = components["schemas"];
type Veto = Schemas["VetoView"];

/** O que tem som no veto. */
export type VetoSoundEvent =
  | { kind: "begin" }
  | { kind: "mapBan"; map: string }
  | { kind: "mapPick"; map: string }
  | { kind: "vote" }
  | { kind: "voteNow" }
  | { kind: "votePassed"; map: string }
  | { kind: "championBan"; champion: string }
  | { kind: "championPick"; champion: string }
  | { kind: "yourTurn"; action: TurnAction }
  | { kind: "enemyPick" }
  | { kind: "done" };

/** O que quem vê tem de fazer na vez dele (a janela às cegas é um pick). */
export type TurnAction = "ban" | "pick" | "vote";

/**
 * O que a comparação guarda de um veto, só em valores simples (a sala de veto
 * monta um objeto novo a cada renderização).
 */
export type VetoSoundState = {
  /** Eventos no histórico, que só cresce (encolhe se o veto for refeito). */
  history: number;
  /** Picks às cegas ainda escondidos, dos outros lados. */
  hidden: number;
  status: Veto["status"];
  /** A votação aberta (a partida), ou `null`. */
  vote: string | null;
  /**
   * A vez de quem joga (`passo:mine:ação`, ou `passo:theirs:pick` na vez de
   * pick do adversário), ou `null`.
   */
  turn: string | null;
  /** As votações já decididas (`partida:mapa`). */
  decided: string[];
};

/** Evento mais velho que isto (pela hora da API na resposta) não toca: a aba que volta não recebe o que perdeu. */
const FRESH_MS = 15_000;

export function soundState(
  veto: Veto,
  mine: Schemas["Side"] | null,
): VetoSoundState {
  const turn = veto.status === "in_progress" ? veto.turn : null;
  return {
    history: veto.history.length,
    hidden: veto.games.reduce((sum, g) => sum + g.hidden_picks.length, 0),
    status: veto.status,
    vote:
      turn?.action === "vote" && turn.sides.length > 0
        ? String(veto.game ?? 1)
        : null,
    turn: turnOf(veto, mine),
    decided: veto.games
      .filter((g) => g.offered.length > 1 && g.map)
      .map((g) => `${g.game}:${g.map?.slug}`),
  };
}

/**
 * A vez de quem joga: a dele (ban, pick ou voto), se pode agir agora, ou a de
 * pick do adversário (a de ban não tem som). Na janela às cegas, os dois
 * escolhem juntos: só a dele conta.
 */
function turnOf(veto: Veto, mine: Schemas["Side"] | null): string | null {
  const turn = veto.turn;
  if (veto.status !== "in_progress" || !turn || mine == null) return null;
  if (isActing(veto, mine)) {
    const action = turn.action === "team_pick" ? "pick" : turn.action;
    return `${veto.step}:mine:${action}`;
  }
  if (turn.action !== "pick" && turn.action !== "team_pick") return null;
  if (isWindow(turn) || turn.sides.length === 0 || turn.sides.includes(mine))
    return null;
  return `${veto.step}:theirs:pick`;
}

/** O som da vez guardada em `VetoSoundState.turn`. */
function turnEvent(turn: string): VetoSoundEvent {
  const [, who, action] = turn.split(":");
  return who === "mine"
    ? { kind: "yourTurn", action: action as TurnAction }
    : { kind: "enemyPick" };
}

/**
 * Os sons entre dois estados do mesmo veto, na ordem em que tocam: os eventos
 * novos do histórico (o passo de cada um, já confirmado pela API), o
 * resultado da votação, o fim do veto e, por último, a votação e a vez que
 * abriram (a de quem vota antes do "vote now", para o efeito da vez tocar
 * junto dele). `now` é a hora da API na resposta mais nova.
 */
export function soundEvents(
  before: VetoSoundState,
  veto: Veto,
  after: VetoSoundState,
  now: number,
): VetoSoundEvent[] {
  const events: VetoSoundEvent[] = [];
  // Na revelação às cegas, os picks entram com a hora em que foram feitos.
  const revealed = before.hidden > 0 && after.hidden === 0;
  for (const event of veto.history.slice(before.history)) {
    const reveal = revealed && event.kind === "pick" && event.champion;
    if (!reveal && Date.parse(event.at) < now - FRESH_MS) continue;
    const found = fromHistory(event);
    if (found) events.push(found);
  }
  for (const decided of after.decided) {
    if (before.decided.includes(decided)) continue;
    events.push({ kind: "votePassed", map: decided.split(":")[1] });
  }
  if (before.status === "in_progress" && after.status === "done") {
    events.push({ kind: "done" });
  }
  const turn =
    after.turn && after.turn !== before.turn ? turnEvent(after.turn) : null;
  const voting = turn?.kind === "yourTurn" && turn.action === "vote";
  if (turn && voting) events.push(turn);
  if (after.vote && after.vote !== before.vote)
    events.push({ kind: "voteNow" });
  if (turn && !voting) events.push(turn);
  return events;
}

function fromHistory(event: Schemas["VetoEvent"]): VetoSoundEvent | null {
  const { kind, map, champion } = event;
  if (kind === "begin") return { kind: "begin" };
  if (kind === "vote") return { kind: "vote" };
  if (kind === "ban" && map) return { kind: "mapBan", map: map.slug };
  if (kind === "pick" && map) return { kind: "mapPick", map: map.slug };
  if (kind === "ban" && champion) {
    return { kind: "championBan", champion: champion.slug };
  }
  if (kind === "pick" && champion) {
    return { kind: "championPick", champion: champion.slug };
  }
  return null;
}
