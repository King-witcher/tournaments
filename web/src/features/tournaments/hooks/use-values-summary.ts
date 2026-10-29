import { useLocale, useTranslations } from "next-intl";
import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { poolOf, type TournamentValues } from "../lib/form-values";

/**
 * O resumo dos valores do formulário, uma linha por grupo: o que aparece no
 * título dos grupos fechados e na revisão antes de publicar. Usa os mesmos
 * textos do resumo do formato na página do torneio.
 */
export function useValuesSummary() {
  const t = useTranslations("TournamentForm.summary");
  const tForm = useTranslations("TournamentForm");
  const tSetup = useTranslations("Tournament.setup");
  const locale = useLocale();
  const formatMinutes = useFormatMinutes();
  const line = (parts: (string | undefined)[]) => {
    const text = parts.filter(Boolean).join(" · ");
    return text.charAt(0).toLocaleUpperCase(locale) + text.slice(1);
  };

  return {
    format: (v: TournamentValues) => {
      const kind =
        v.stageFormat !== "elimination"
          ? v.stageFormat
          : v.bracket === "double" && v.grandFinalReset
            ? "doubleReset"
            : v.bracket;
      return line([
        tSetup(`bracket.${kind}`),
        v.stageFormat === "elimination"
          ? tSetup(`series.${v.bracket}`, {
              bestOf: v.bestOf,
              finals: v.finalsBestOf,
            })
          : tSetup("series.table", { bestOf: v.bestOf }),
        v.stageFormat === "swiss"
          ? v.swissRounds === null
            ? tSetup("rounds.auto")
            : tSetup("rounds.fixed", { count: v.swissRounds })
          : undefined,
      ]);
    },
    registration: (v: TournamentValues) =>
      line([
        tForm(`visibilityOptions.${v.visibility}`),
        v.maxParticipants === null
          ? undefined
          : t("slots", { count: v.maxParticipants }),
        v.checkInMinutes === null
          ? t("noCheckIn")
          : t("checkIn", { window: formatMinutes(v.checkInMinutes) }),
        t("contest", { window: formatMinutes(v.contestMinutes) }),
      ]),
    champions: (v: TournamentValues) =>
      line([
        tSetup("bans", { count: v.bansPerGame }),
        tSetup(`noRepeat.${v.noRepeat}`),
      ]),
    veto: (v: TournamentValues, maps: components["schemas"]["Map"][]) =>
      line([
        tForm(`veto.mapModes.${v.mapMode}`),
        v.mapMode === "off"
          ? undefined
          : t("maps", { count: poolOf(v, maps).length }),
        v.championPhase ? t("championPhase") : t("noChampionPhase"),
        v.mapMode !== "off" || v.championPhase
          ? t("seconds", { seconds: v.vetoSeconds })
          : undefined,
      ]),
  };
}
