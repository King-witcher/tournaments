import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { formatKind, repeatScope } from "../lib/format";
import { useFormatSeconds } from "./use-format-seconds";

type Tournament = components["schemas"]["Tournament"];

/**
 * As frases das regras que a aba Regras ("Como funciona") e os (i) do resumo
 * do formato dividem, para os dois dizerem o mesmo: a regra da chave, os
 * campeões e, no TDM, os times. No TDM, as frases dizem time e capitão onde o
 * duelo diz jogador.
 */
export function useRuleSentences() {
  const t = useTranslations("Tournament.autoRules");
  const formatSeconds = useFormatSeconds();

  return {
    /** Como a chave (ou o round robin, o suíço) leva ao campeão. */
    bracketRule: (tournament: Tournament) => {
      const kind = formatKind(tournament.stages[0]);
      return tournament.mode === "tdm" && kind === "round_robin"
        ? t("tdm.roundRobinRule")
        : t(`bracketRule.${kind}`);
    },

    /**
     * Bans e não repetição; no TDM, também quem bane (o capitão), a escolha
     * simultânea (com a fase de campeões) e campeão igual no time.
     */
    champions: (tournament: Tournament): string[] => {
      const rules = tournament.champion_rules;
      const bans = rules.bans_per_game;
      const scope = repeatScope(rules);
      if (tournament.mode !== "tdm") {
        return [
          bans > 0 ? t("champions.bans", { count: bans }) : t("noBans"),
          t(`champions.repeat.${scope}`),
        ];
      }
      const who = rules.no_repeat?.who ?? "player";
      return [
        bans > 0 ? t("tdm.bans", { count: bans }) : t("noBans"),
        ...(tournament.veto.champion_phase
          ? [
              t("tdm.pick", {
                window: formatSeconds(tournament.veto.team_pick_seconds),
              }),
            ]
          : []),
        t(rules.team_duplicates ? "tdm.duplicates.on" : "tdm.duplicates.off"),
        scope === "off" ? t("tdm.repeat.off") : t(`tdm.repeat.${who}.${scope}`),
      ];
    },

    /**
     * TDM: a formação dos times, os reservas, o que o início exige e o que
     * o capitão faz (só com alguma fase de veto). Vazio no duelo.
     */
    teams: (tournament: Tournament): string[] => {
      if (tournament.mode !== "tdm") return [];
      const veto = tournament.veto;
      return [
        t("tdm.formation", { size: tournament.team_size }),
        t("tdm.reserves"),
        t(
          tournament.check_in_minutes == null
            ? "tdm.startNoCheckIn"
            : "tdm.start",
        ),
        ...(veto.map_mode !== "off" || veto.champion_phase
          ? [t("tdm.captain")]
          : []),
      ];
    },
  };
}
