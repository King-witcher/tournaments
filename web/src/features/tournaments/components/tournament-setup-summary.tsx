import { Stack } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { Href } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { InfoTip } from "@/shared/ui/info-tip";
import { Link } from "@/shared/ui/link";
import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";
import { useModeLabel } from "../hooks/use-mode-label";
import { useRuleSentences } from "../hooks/use-rule-sentences";
import { useValuesSummary } from "../hooks/use-values-summary";
import { valuesFrom } from "../lib/form-values";
import { formatKind, repeatScope, teamsIn } from "../lib/format";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** A aba Regras, que o link embaixo do resumo abre. */
  rulesHref: Href;
};

type Tip = "mode" | "bracket" | "format" | "champions" | "checkIn" | "contest";

/**
 * O formato, em rótulo e valor curto: modalidade, chave (ou round robin,
 * suíço), séries, rodadas do suíço, vagas (no TDM, com os times que cabem),
 * regras de campeões e o veto da fase única, as janelas de check-in e de
 * contestação e se conta para o rating da comunidade. Os termos que quem
 * joga pode não conhecer têm um (i) com a explicação (a dos times, a da
 * chave e a dos campeões são as mesmas frases da aba Regras). Embaixo, o
 * link para a aba Regras, onde está tudo por extenso.
 */
export function TournamentSetupSummary({ tournament, rulesHref }: Props) {
  const t = useTranslations("Tournament.setup");
  const locale = useLocale();
  const tRating = useTranslations("Ratings.tournament");
  const formatMinutes = useFormatMinutes();
  const modeLabel = useModeLabel();
  const rules = useRuleSentences();
  const summary = useValuesSummary();
  // Campeões e veto com o mesmo texto do resumo do formulário.
  const values = valuesFrom(tournament);
  const tdm = tournament.mode === "tdm";
  const [stage] = tournament.stages;
  const kind = formatKind(stage);
  const bans = tournament.champion_rules.bans_per_game;
  const repeat = repeatScope(tournament.champion_rules);
  const checkIn = tournament.check_in_minutes;
  const slots = tournament.max_participants;
  // O valor junta pedaços em minúscula ("abre 30 min antes do início").
  const value = (text: string) =>
    text.charAt(0).toLocaleUpperCase(locale) + text.slice(1);
  const tip = (key: Tip, text: string) => (
    <InfoTip term={t(`labels.${key}`)} label={t(`tips.${key}`)}>
      {text}
    </InfoTip>
  );
  const checkInTip = tdm
    ? checkIn == null
      ? "tips.noCheckInTextTdm"
      : "tips.checkInTextTdm"
    : checkIn == null
      ? "tips.noCheckInText"
      : "tips.checkInText";

  const rows: { key: string; label: ReactNode; value: string }[] = [
    {
      key: "mode",
      // No duelo, "Duelo" não tem o que explicar.
      label: tdm
        ? tip("mode", rules.teams(tournament).join(" "))
        : t("labels.mode"),
      value: modeLabel(tournament.mode, tournament.team_size),
    },
    {
      key: "bracket",
      // "Chave" só na eliminação; round robin e suíço são "Formato".
      label: tip(
        stage.format === "elimination" ? "bracket" : "format",
        rules.bracketRule(tournament),
      ),
      value: t(`bracket.${kind}`),
    },
    {
      key: "series",
      label: t("labels.series"),
      value:
        stage.format === "elimination"
          ? t(`series.${stage.bracket}`, {
              bestOf: stage.best_of,
              finals: stage.finals_best_of,
            })
          : t("series.table", { bestOf: stage.best_of }),
    },
    ...(stage.format === "swiss"
      ? [
          {
            key: "rounds",
            label: t("labels.rounds"),
            // No TDM, as rodadas contam times, não inscritos.
            value:
              stage.rounds == null
                ? t(tdm ? "roundsValue.autoTeams" : "roundsValue.auto")
                : t(tdm ? "roundsValue.fixedTeams" : "roundsValue.fixed", {
                    count: stage.rounds,
                  }),
          },
        ]
      : []),
    {
      key: "slots",
      label: t("labels.slots"),
      value: tdm
        ? t("slotsTeams", {
            slots,
            size: tournament.team_size,
            teams: teamsIn(slots, tournament.team_size).teams,
          })
        : String(slots),
    },
    {
      key: "champions",
      // No duelo, sem ban e sem regra, "Podem repetir" não tem o que
      // explicar; no TDM, há sempre a escolha e o campeão igual no time.
      label:
        tdm || bans > 0 || repeat !== "off"
          ? tip("champions", rules.champions(tournament).join(" "))
          : t("labels.champions"),
      value: summary.champions(values),
    },
    { key: "veto", label: t("labels.veto"), value: summary.veto(values, []) },
    {
      key: "checkIn",
      label: tip("checkIn", t(checkInTip)),
      value: value(
        checkIn == null
          ? t("noCheckIn")
          : t("checkIn", { window: formatMinutes(checkIn) }),
      ),
    },
    {
      key: "contest",
      label: tip(
        "contest",
        t(tdm ? "tips.contestTextTdm" : "tips.contestText"),
      ),
      value: formatMinutes(tournament.contest_minutes),
    },
    {
      key: "rating",
      label: tRating("label"),
      value: tournament.counts_for_rating
        ? tRating("counts")
        : tRating("doesNotCount"),
    },
  ];

  return (
    <Section title={t("title")}>
      <Stack gap={5}>
        <StructuredList
          label={t("title")}
          condensed
          rows={rows.map((row) => ({
            key: row.key,
            cells: [row.label, row.value],
          }))}
        />
        {/* Na largura do texto: solto na pilha, o link ocuparia a linha toda. */}
        <div>
          <Link href={rulesHref}>{t("seeRules")}</Link>
        </div>
      </Stack>
    </Section>
  );
}
