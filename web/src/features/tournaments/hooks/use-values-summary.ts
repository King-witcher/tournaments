import { useLocale, useTranslations } from "next-intl";
import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import {
  isTdm,
  poolOf,
  stepSecondsGroup,
  type TournamentValues,
  teamSizeOf,
} from "../lib/form-values";
import { teamsIn } from "../lib/format";
import { useFormatSeconds } from "./use-format-seconds";
import { useModeLabel } from "./use-mode-label";

/**
 * O resumo dos valores do formulário, uma linha por grupo: o que aparece no
 * título dos grupos fechados e na revisão antes de publicar (e, a partir de
 * um torneio salvo, no resumo do formato da página dele). Usa os mesmos
 * textos do resumo do formato na página do torneio.
 */
export function useValuesSummary() {
  const t = useTranslations("TournamentForm.summary");
  const tForm = useTranslations("TournamentForm");
  const tSetup = useTranslations("Tournament.setup");
  const locale = useLocale();
  const formatMinutes = useFormatMinutes();
  const formatSeconds = useFormatSeconds();
  const modeLabel = useModeLabel();
  const line = (parts: (string | false | undefined)[]) => {
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
        modeLabel(v.mode, teamSizeOf(v)),
        tSetup(`bracket.${kind}`),
        v.stageFormat === "elimination"
          ? tSetup(`series.${v.bracket}`, {
              bestOf: v.bestOf,
              finals: v.finalsBestOf,
            })
          : tSetup("series.table", { bestOf: v.bestOf }),
        v.stageFormat === "swiss"
          ? v.swissRounds === null
            ? tSetup(isTdm(v) ? "rounds.autoTeams" : "rounds.auto")
            : tSetup("rounds.fixed", { count: v.swissRounds })
          : undefined,
      ]);
    },
    registration: (v: TournamentValues) => {
      const slots = v.maxParticipants;
      const size = teamSizeOf(v);
      return line([
        tForm(`admissionOptions.${v.admission}`),
        slots !== null &&
          (isTdm(v)
            ? t("slotsTeams", {
                count: slots,
                size,
                teams: teamsIn(slots, size).teams,
              })
            : t("slots", { count: slots })),
        v.checkInMinutes === null
          ? t("noCheckIn")
          : t("checkIn", { window: formatMinutes(v.checkInMinutes) }),
      ]);
    },
    /** `screenshots`: a API tem capturas de tela; sem elas, o campo não aparece e o valor não conta. */
    results: (v: TournamentValues, screenshots: boolean) =>
      line([
        t("contest", { window: formatMinutes(v.contestMinutes) }),
        screenshots && v.screenshotsRequired && t("screenshotsRequired"),
      ]),
    champions: (v: TournamentValues) => {
      const repeat = tSetup(`noRepeat.${v.noRepeat}`);
      const tdm = isTdm(v);
      return line([
        !v.championPhase && tSetup("offSite"),
        tSetup("bans", { count: v.bansPerSeries }),
        !v.mirror && tSetup("noMirror"),
        tdm && v.sameTeam && tSetup("sameTeam"),
        tdm && v.noRepeat !== "off"
          ? tSetup("noRepeatWho", { rule: repeat, who: v.noRepeatWho })
          : repeat,
        stepSecondsGroup(v) === "champions" &&
          t("seconds", { time: formatSeconds(v.vetoSeconds) }),
        tdm &&
          v.championPhase &&
          t("teamPick", { time: formatSeconds(v.teamPickSeconds) }),
      ]);
    },
    veto: (v: TournamentValues, maps: components["schemas"]["Map"][]) =>
      line([
        tForm(`veto.mapModes.${v.mapMode}`),
        v.mapMode !== "off" && t("maps", { count: poolOf(v, maps).length }),
        stepSecondsGroup(v) === "maps" &&
          t("seconds", { time: formatSeconds(v.vetoSeconds) }),
      ]),
  };
}
