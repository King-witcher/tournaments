import { mapSequence } from "@/features/tournaments/lib/veto-sequence";
import type { components } from "@/shared/api/schema";
import { otherSide } from "./veto";

type Schemas = components["schemas"];
type Veto = Schemas["VetoView"];
type Turn = Schemas["VetoTurn"];
type Champion = Schemas["ChampionRef"];

/** O passo que quem vê manda: um mapa, ou um campeão (na janela do TDM, com a partida). */
export type VetoChoice = { map: string } | { champion: string; game?: number };

/** O que o passo otimista precisa saber além do veto. */
export type StepContext = {
  mine: Schemas["Side"];
  bestOf: number;
  /** Bans de campeão de cada lado na série (do torneio). */
  bansPerSeries: number;
  /** A ordem do veto de mapas (do torneio). */
  order: Schemas["VetoOrder"];
  /** Quem vê, para o pick da janela do TDM. */
  viewer: { id: number; nickname: string } | null;
  now: number;
};

/**
 * O veto como fica logo depois do passo de quem vê, antes da resposta da
 * API: o mapa ou o campeão marcado, as opções dele somem, desfazer espera e a
 * vez passa. A vez seguinte só sai quando é certa (dentro do veto de mapas,
 * dos bans ou dos picks); na troca de fase, a vez fica a mesma sem ninguém
 * (\`sides\` vazio), para a tela não pular de passo até a resposta, que traz
 * a vez de verdade e substitui tudo isto.
 */
export function afterStep(
  veto: Veto,
  choice: VetoChoice,
  ctx: StepContext,
): Veto {
  const turn = veto.turn;
  if (!turn) return veto;
  const base: Veto = {
    ...veto,
    options: [],
    undo_request: null,
    can_undo: false,
  };
  if ("map" in choice) {
    return turn.action === "vote"
      ? afterVote(base, turn, choice.map, ctx)
      : afterMap(base, turn, choice.map, ctx);
  }
  const found = veto.champions.find((c) => c.slug === choice.champion);
  if (!found) return base;
  const champion = { slug: found.slug, name: found.name };
  if (turn.action === "ban") return afterBan(base, turn, champion, ctx);
  if (turn.action === "pick") return afterPick(base, turn, champion, ctx);
  return afterTeamPick(base, champion, choice.game, ctx);
}

/** A vez seguinte, com um tempo inteiro a partir de agora; sem ela, a troca de fase. */
function pass(veto: Veto, turn: Turn, next: Turn | null, ctx: StepContext) {
  // Sem o segundo que o arredondamento do cronômetro somaria.
  const deadline = new Date(ctx.now + veto.step_seconds * 1000 - 999);
  return next
    ? { ...veto, turn: next, deadline: deadline.toISOString() }
    : { ...veto, turn: { ...turn, sides: [] }, deadline: null };
}

function afterMap(veto: Veto, turn: Turn, slug: string, ctx: StepContext) {
  const map = veto.pool.find((m) => m.slug === slug);
  if (!map) return veto;
  const steps = [
    ...veto.map_steps,
    { action: turn.action, map, side: ctx.mine },
  ];
  const sequence = mapSequence(veto.pool.length, ctx.bestOf, ctx.order) ?? [];
  const action = sequence[steps.length];
  const next = action
    ? { ...turn, action, sides: [otherSide(ctx.mine)] }
    : null;
  return {
    ...pass(veto, turn, next, ctx),
    map_steps: steps,
    remaining: veto.remaining.filter((m) => m.slug !== slug),
  };
}

/** O voto: o outro lado ainda vota no mesmo prazo; com os dois, a troca de fase. */
function afterVote(veto: Veto, turn: Turn, slug: string, ctx: StepContext) {
  const game = veto.game ?? 1;
  const left = turn.sides.filter((s) => s !== ctx.mine);
  return {
    ...veto,
    turn: { ...turn, sides: left },
    games: veto.games.map((g) =>
      g.game === game ? { ...g, [`vote_${ctx.mine}`]: slug } : g,
    ),
  };
}

function afterBan(
  veto: Veto,
  turn: Turn,
  champion: Champion,
  ctx: StepContext,
) {
  const bans = [...veto.bans, { champion, side: ctx.mine, timeout: false }];
  const next =
    bans.length < 2 * ctx.bansPerSeries
      ? { ...turn, sides: [otherSide(ctx.mine)] }
      : null;
  return {
    ...pass(veto, turn, next, ctx),
    bans,
    champions: veto.champions.map((c) =>
      c.slug === champion.slug ? { ...c, banned_by: ctx.mine } : c,
    ),
  };
}

/** Pick do duelo: quem escolhe por último numa partida escolhe primeiro na seguinte. */
function afterPick(
  veto: Veto,
  turn: Turn,
  champion: Champion,
  ctx: StepContext,
) {
  const game = turn.pick_for ?? 1;
  const other = otherSide(ctx.mine);
  const second = veto.games.some((g) => g.game === game && g[`pick_${other}`]);
  const next = !second
    ? { ...turn, sides: [other] }
    : game < ctx.bestOf
      ? { ...turn, sides: [ctx.mine], pick_for: game + 1 }
      : null;
  return {
    ...pass(veto, turn, next, ctx),
    picks: [],
    games: veto.games.map((g) =>
      g.game === game ? { ...g, [`pick_${ctx.mine}`]: champion } : g,
    ),
  };
}

/** Janela simultânea do TDM: a partida escolhida sai do que falta; a vez fica. */
function afterTeamPick(
  veto: Veto,
  champion: Champion,
  game: number | undefined,
  ctx: StepContext,
) {
  const viewer = ctx.viewer;
  if (game == null || !viewer) return veto;
  const pick = {
    champion,
    side: ctx.mine,
    nickname: viewer.nickname,
    user_id: viewer.id,
    timeout: false,
  };
  return {
    ...veto,
    picks: veto.picks.filter((p) => p.game !== game),
    missing: veto.missing
      .map((p) =>
        p.user_id === viewer.id
          ? { ...p, games: p.games.filter((g) => g !== game) }
          : p,
      )
      .filter((p) => p.games.length > 0),
    games: veto.games.map((g) =>
      g.game === game ? { ...g, team_picks: [...g.team_picks, pick] } : g,
    ),
  };
}
