import { useLocale, useTranslations } from "next-intl";
import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import {
  hasPickWindow,
  isTdm,
  onlyEvenSeries,
  poolOf,
  stepSecondsGroup,
  type TournamentValues,
  teamSizeOf,
} from "../lib/form-values";
import { teamsIn } from "../lib/format";
import type { StageValues } from "../lib/stage-values";
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

  /** O formato de uma fase, com o reset da grande final no nome. */
  const stageLine = (s: StageValues, tdm: boolean) => {
    const kind =
      s.format !== "elimination"
        ? s.format
        : s.bracket === "double" && s.grandFinalReset
          ? "doubleReset"
          : s.bracket;
    return line([
      tSetup(`bracket.${kind}`),
      s.format === "elimination"
        ? tSetup(`series.${s.bracket}`, {
            bestOf: s.bestOf,
            finals: s.finalsBestOf,
          })
        : tSetup("series.table", { bestOf: s.bestOf }),
      s.format === "swiss"
        ? s.swissRounds === null
          ? tSetup(tdm ? "rounds.autoTeams" : "rounds.auto")
          : tSetup("rounds.fixed", { count: s.swissRounds })
        : undefined,
    ]);
  };

  /** Uma fase em poucas palavras, para o título com várias: "Todos contra todos, md2". */
  const stagePhrase = (s: StageValues) =>
    `${tSetup(`bracket.${s.format === "elimination" ? s.bracket : s.format}`)}, ${
      s.format === "elimination"
        ? t("seriesFinals", { bestOf: s.bestOf, finals: s.finalsBestOf })
        : t("series", { bestOf: s.bestOf })
    }`;

  /** Modalidade e vagas. */
  const base = (v: TournamentValues) => {
    const slots = v.maxParticipants;
    const size = teamSizeOf(v);
    return [
      modeLabel(v.mode, size),
      slots !== null &&
        (isTdm(v)
          ? t("slotsTeams", {
              count: slots,
              size,
              teams: teamsIn(slots, size).teams,
            })
          : t("slots", { count: slots })),
    ];
  };

  return {
    /**
     * Modalidade, vagas e as fases: com uma, o formato dela por inteiro; com
     * várias, uma frase curta por fase, ligadas por setas.
     */
    format: (v: TournamentValues) =>
      line([
        ...base(v),
        v.stages.length === 1
          ? stageLine(v.stages[0], isTdm(v))
          : v.stages.map(stagePhrase).join(" → "),
      ]),
    /** Só modalidade e vagas: as fases vêm à parte (a revisão com várias). */
    mode: (v: TournamentValues) => line(base(v)),
    /** O formato de uma fase por inteiro (bracket, séries e rodadas). */
    stage: (s: StageValues, v: Pick<TournamentValues, "mode">) =>
      stageLine(s, isTdm(v)),
    /**
     * Os desempates que contam, na ordem ("Confronto direto, partidas, frags"),
     * ou "Só a ordem da chave"; com a organização decidindo o empate do fim, ela
     * por último. Vazio na eliminação.
     */
    tiebreakers: (
      v: Pick<StageValues, "format" | "bestOf" | "tiebreakers" | "finalTie">,
    ) => {
      if (!isTableFormat(v.format)) return "";
      const list = countingTiebreakers(v.tiebreakers, v.format, v.bestOf);
      const organizer = v.finalTie === "organizer";
      if (list.length === 0) {
        return organizer ? tTie("organizerOnly") : tTie("none");
      }
      const names = list.map((x) => tTie(`short.${x}`));
      if (organizer) names.push(tTie("short.organizer"));
      return upper(names.join(", "));
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
          !onlyEvenSeries(v) &&
          t(`vetoOrder.${v.vetoOrder}`),
        v.mapMode !== "off" && t("maps", { count: poolOf(v, maps).length }),
        stepSecondsGroup(v) === "maps" &&
          t("seconds", { time: formatSeconds(v.vetoSeconds) }),
      ]),
  };
}
