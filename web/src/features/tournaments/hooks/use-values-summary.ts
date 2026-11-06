import { useLocale, useTranslations } from "next-intl";
import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import {
  hasPickWindow,
  isTdm,
  poolOf,
  stepSecondsGroup,
  type TournamentValues,
  teamSizeOf,
} from "../lib/form-values";
import { isEvenSeries, teamsIn } from "../lib/format";
import { teamPresetOf } from "../lib/team-rules";
import { countingTiebreakers, isTableFormat } from "../lib/tiebreakers";
import { useFormatSeconds } from "./use-format-seconds";
import { useModeLabel } from "./use-mode-label";

/**
 * O resumo dos valores de um torneio, uma linha por grupo: na revisão antes
 * de publicar, no resumo do formato da página dele e no título dos grupos
 * fechados da sala de veto. Usa os mesmos textos do resumo do formato na
 * página do torneio.
 */
export function useValuesSummary() {
  const t = useTranslations("TournamentForm.summary");
  const tForm = useTranslations("TournamentForm");
  const tSetup = useTranslations("Tournament.setup");
  const tTeam = useTranslations("TournamentForm.teamRules");
  const tTie = useTranslations("TournamentForm.tiebreakers");
  const locale = useLocale();
  const formatMinutes = useFormatMinutes();
  const formatSeconds = useFormatSeconds();
  const modeLabel = useModeLabel();
  const upper = (text: string) =>
    text.charAt(0).toLocaleUpperCase(locale) + text.slice(1);
  const line = (parts: (string | false | undefined)[]) =>
    upper(parts.filter(Boolean).join(" · "));

  return {
    format: (v: TournamentValues) => {
      const kind =
        v.stageFormat !== "elimination"
          ? v.stageFormat
          : v.bracket === "double" && v.grandFinalReset
            ? "doubleReset"
            : v.bracket;
      const slots = v.maxParticipants;
      const size = teamSizeOf(v);
      return line([
        modeLabel(v.mode, size),
        slots !== null &&
          (isTdm(v)
            ? t("slotsTeams", {
                count: slots,
                size,
                teams: teamsIn(slots, size).teams,
              })
            : t("slots", { count: slots })),
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
    /**
     * Os desempates que contam, na ordem ("Confronto direto, partidas, frags"),
     * ou "Só a ordem da chave"; vazio na eliminação.
     */
    tiebreakers: (
      v: Pick<TournamentValues, "stageFormat" | "bestOf" | "tiebreakers">,
    ) => {
      if (!isTableFormat(v.stageFormat)) return "";
      const list = countingTiebreakers(v.tiebreakers, v.stageFormat, v.bestOf);
      return list.length === 0
        ? tTie("none")
        : upper(list.map((x) => tTie(`short.${x}`)).join(", "));
    },
    /** Quem age pelo time: o preset que as regras formam. */
    teamRules: (v: Pick<TournamentValues, "teamRules">) =>
      tTeam(`presets.${teamPresetOf(v.teamRules)}`),
    registration: (v: TournamentValues) =>
      line([
        tForm(`admissionOptions.${v.admission}`),
        v.checkInMinutes === null
          ? t("noCheckIn")
          : t("checkIn", { window: formatMinutes(v.checkInMinutes) }),
      ]),
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
        v.championPhase && tSetup(`pickOrder.${v.pickOrder}`),
        !v.mirror && tSetup("noMirror"),
        tdm && v.sameTeam && tSetup("sameTeam"),
        tdm && v.noRepeat !== "off"
          ? tSetup("noRepeatWho", { rule: repeat, who: v.noRepeatWho })
          : repeat,
        stepSecondsGroup(v) === "champions" &&
          t("seconds", { time: formatSeconds(v.vetoSeconds) }),
        hasPickWindow(v) &&
          t("teamPick", { time: formatSeconds(v.teamPickSeconds) }),
      ]);
    },
    veto: (v: TournamentValues, maps: components["schemas"]["Map"][]) =>
      line([
        tForm(`veto.mapModes.${v.mapMode}`),
        v.mapMode === "veto" &&
          !isEvenSeries(v.bestOf) &&
          t(`vetoOrder.${v.vetoOrder}`),
        v.mapMode !== "off" && t("maps", { count: poolOf(v, maps).length }),
        stepSecondsGroup(v) === "maps" &&
          t("seconds", { time: formatSeconds(v.vetoSeconds) }),
      ]),
  };
}
