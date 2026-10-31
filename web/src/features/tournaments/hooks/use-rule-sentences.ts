import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { formatKind, repeatScope } from "../lib/format";
import { useFormatSeconds } from "./use-format-seconds";

type Tournament = components["schemas"]["Tournament"];

/**
 * As frases das regras que a aba Regras ("Como funciona") e os (i) do resumo
 * do formato dividem, para os dois dizerem o mesmo: a regra da chave, os
 * campeões e, no TDM, os times. No TDM, as frases dizem time onde o duelo
 * diz jogador, e nunca falam de time no duelo.
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
     * O draft de campeões da série: os bans, os picks (no duelo, partida a
     * partida; no TDM, a janela simultânea, com o draft no site), o que pode
     * repetir na partida e entre partidas. No TDM, qualquer jogador age pelo
     * time.
     */
    champions: (tournament: Tournament): string[] => {
      const rules = tournament.champion_rules;
      const bans = rules.bans_per_series;
      const scope = repeatScope(rules);
      const site = tournament.veto.champion_phase;
      const tdm = tournament.mode === "tdm";
      const who = rules.no_repeat?.who ?? "player";
      const mode = tdm ? "tdm" : "duel";
      return [
        bans > 0 ? t(`champions.bans.${mode}`, { count: bans }) : t("noBans"),
        site &&
          (tdm
            ? t("champions.pick.tdm", {
                window: formatSeconds(tournament.veto.team_pick_seconds),
              })
            : t("champions.pick.duel")),
        t(`champions.mirror.${mode}.${rules.mirror ? "on" : "off"}`),
        tdm &&
          t(
            rules.same_team
              ? "champions.sameTeam.on"
              : "champions.sameTeam.off",
          ),
        scope === "off"
          ? t("champions.repeat.off")
          : tdm
            ? t(`champions.repeat.${who}.${scope}`)
            : t(`champions.repeat.duel.${scope}`),
      ].filter((s): s is string => typeof s === "string");
    },

    /**
     * TDM: o tamanho dos times, quem forma, os reservas e as trocas durante
     * o torneio. Vazio no duelo.
     */
    teams: (tournament: Tournament): string[] => {
      if (tournament.mode !== "tdm") return [];
      return [
        t("teams.formation", { size: tournament.team_size }),
        t("teams.changes"),
      ];
    },
  };
}
