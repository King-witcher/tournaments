import type { components } from "@/shared/api/schema";

type TeamRules = components["schemas"]["TeamRules"];

/** O preset do formulário: a API só guarda as regras, e ele sai delas. */
export type TeamPreset = "captain" | "balanced" | "everyone" | "custom";

/** Os presets que têm valores (Personalizado é o resto). */
export const TEAM_PRESETS: Record<Exclude<TeamPreset, "custom">, TeamRules> = {
  captain: {
    veto: "captain",
    ban: "captain",
    pick: "captain",
    report: "captain",
  },
  balanced: {
    veto: "captain",
    ban: "captain",
    pick: "each_player",
    report: "anyone",
  },
  everyone: {
    veto: "anyone",
    ban: "anyone",
    pick: "each_player",
    report: "anyone",
  },
};

const same = (a: TeamRules, b: TeamRules) =>
  a.veto === b.veto &&
  a.ban === b.ban &&
  a.pick === b.pick &&
  a.report === b.report;

/**
 * Só o capitão tem o que fazer antes das partidas: nenhum passo que o torneio
 * tem (veto ou votação de mapa, bans, escolha de campeões) é aberto ao time.
 * É ele, então, quem marca "Pronto". Enviar o resultado vem depois e não
 * entra. A mesma regra de `Stage::captain_acts_alone` na API.
 */
export function captainActsAlone(
  rules: TeamRules,
  setup: { mapMode: string; championPhase: boolean; bansPerSeries: number },
): boolean {
  const maps = setup.mapMode === "veto" || setup.mapMode === "vote";
  const bans = setup.championPhase && setup.bansPerSeries > 0;
  return !(
    (maps && rules.veto === "anyone") ||
    (bans && rules.ban === "anyone") ||
    (setup.championPhase && rules.pick === "each_player")
  );
}

/** O preset que as regras formam; nenhum, Personalizado. */
export function teamPresetOf(rules: TeamRules): TeamPreset {
  if (same(rules, TEAM_PRESETS.captain)) return "captain";
  if (same(rules, TEAM_PRESETS.balanced)) return "balanced";
  if (same(rules, TEAM_PRESETS.everyone)) return "everyone";
  return "custom";
}
