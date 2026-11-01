import type { components } from "@/shared/api/schema";
import { mapSequence, otherSide } from "./veto";

type Schemas = components["schemas"];
type Veto = Schemas["VetoView"];
type Turn = Schemas["VetoTurn"];

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
 * API: o mapa ou o campeão marcado, as opções dele somem e a vez passa. A
 * vez seguinte só sai quando é certa (dentro do veto de mapas, dos bans ou
 * dos picks); na troca de fase fica `null` até a resposta, que traz a vez de
 * verdade e substitui tudo isto.
 */
export function afterStep(
  veto: Veto,
  choice: VetoChoice,
  ctx: StepContext,
): Veto {
  const turn = veto.turn;
  const other = otherSide(ctx.mine);
  // Um tempo inteiro a partir de agora (sem o segundo que o arredondamento
  // do cronômetro somaria); a resposta traz o prazo de verdade.
  const deadline = new Date(
    ctx.now + veto.step_seconds * 1000 - 999,
  ).toISOString();
  const base: Veto = { ...veto, options: [], undo_request: null };
  const pass = (next: Turn | null): Veto => ({
    ...base,
    turn: next,
    deadline: next ? deadline : null,
  });

  if ("map" in choice) {
    const map = veto.pool.find((m) => m.slug === choice.map);
    if (!turn || !map) return base;
    if (turn.action === "vote") {
      const game = veto.game ?? 1;
      const left = turn.sides.filter((s) => s !== ctx.mine);
      return {
        ...base,
        turn: left.length > 0 ? { ...turn, sides: left } : null,
        games: veto.games.map((g) =>
          g.game === game ? { ...g, [`vote_${ctx.mine}`]: map.slug } : g,
        ),
      };
    }
    const steps = [
      ...veto.map_steps,
      { action: turn.action, map, side: ctx.mine },
    ];
    const sequence = mapSequence(veto.pool.length, ctx.bestOf, ctx.order);
    const next = sequence[steps.length];
    return {
      ...pass(next ? { ...turn, action: next, sides: [other] } : null),
      map_steps: steps,
      remaining: veto.remaining.filter((m) => m.slug !== map.slug),
    };
  }

  const found = veto.champions.find((c) => c.slug === choice.champion);
  if (!turn || !found) return base;
  const champion = { slug: found.slug, name: found.name };
  if (turn.action === "ban") {
    const bans = [...veto.bans, { champion, side: ctx.mine, timeout: false }];
    return {
      ...pass(
        bans.length < 2 * ctx.bansPerSeries
          ? { ...turn, sides: [other] }
          : null,
      ),
      bans,
      champions: veto.champions.map((c) =>
        c.slug === champion.slug ? { ...c, banned_by: ctx.mine } : c,
      ),
    };
  }
  if (turn.action === "pick") {
    const game = turn.pick_for ?? 1;
    const second = veto.games.some(
      (g) => g.game === game && g[`pick_${other}`],
    );
    // Quem escolhe por último numa partida escolhe primeiro na seguinte.
    const next = !second
      ? { ...turn, sides: [other] }
      : game < ctx.bestOf
        ? { ...turn, sides: [ctx.mine], pick_for: game + 1 }
        : null;
    return {
      ...pass(next),
      picks: [],
      games: veto.games.map((g) =>
        g.game === game ? { ...g, [`pick_${ctx.mine}`]: champion } : g,
      ),
    };
  }
  // Janela simultânea do TDM: a partida escolhida sai do que falta; a vez fica.
  const game = "game" in choice ? choice.game : undefined;
  const viewer = ctx.viewer;
  if (game == null || !viewer) return base;
  return {
    ...veto,
    options: [],
    picks: veto.picks.filter((p) => p.game !== game),
    missing: veto.missing
      .map((p) =>
        p.user_id === viewer.id
          ? { ...p, games: p.games.filter((g) => g !== game) }
          : p,
      )
      .filter((p) => p.games.length > 0),
    games: veto.games.map((g) =>
      g.game === game
        ? {
            ...g,
            team_picks: [
              ...g.team_picks,
              {
                champion,
                side: ctx.mine,
                nickname: viewer.nickname,
                user_id: viewer.id,
                timeout: false,
              },
            ],
          }
        : g,
    ),
  };
}
