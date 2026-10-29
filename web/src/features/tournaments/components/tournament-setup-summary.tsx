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
import { bracketKind, repeatScope } from "../lib/format";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** A aba Regras, que o link embaixo do resumo abre. */
  rulesHref: Href;
};

/**
 * O formato, em rótulo e valor curto: chave, séries, vagas, regras de
 * campeões e o veto da fase única, as janelas de check-in e de contestação e
 * se conta para o rating da comunidade. Os termos que quem joga pode não
 * conhecer têm um (i) com a explicação (a da chave e a dos campeões são as
 * mesmas frases da aba Regras). Embaixo, o link para a aba Regras, onde está
 * tudo por extenso.
 */
export function TournamentSetupSummary({ tournament, rulesHref }: Props) {
  const t = useTranslations("Tournament.setup");
  const tRules = useTranslations("Tournament.autoRules");
  const locale = useLocale();
  const tRating = useTranslations("Ratings.tournament");
  const tForm = useTranslations("TournamentForm");
  const formatMinutes = useFormatMinutes();
  const [stage] = tournament.stages;
  const bracket = bracketKind(stage);
  const bans = tournament.champion_rules.bans_per_game;
  const repeat = repeatScope(tournament.champion_rules);
  const checkIn = tournament.check_in_minutes;
  // O valor junta pedaços em minúscula ("1 ban por mapa · único por série").
  const value = (...parts: (string | false)[]) => {
    const text = parts.filter(Boolean).join(" · ");
    return text.charAt(0).toLocaleUpperCase(locale) + text.slice(1);
  };
  const tip = (
    key: "bracket" | "champions" | "checkIn" | "contest",
    text: string,
  ) => (
    <InfoTip term={t(`labels.${key}`)} label={t(`tips.${key}`)}>
      {text}
    </InfoTip>
  );
  // O veto numa linha: modo do mapa, mapas no pool, campeões e tempo.
  const v = tournament.veto;
  const vetoText = value(
    tForm(`veto.mapModes.${v.map_mode}`),
    v.map_mode !== "off" && tForm("summary.maps", { count: v.map_pool.length }),
    v.champion_phase
      ? tForm("summary.championPhase")
      : tForm("summary.noChampionPhase"),
    (v.map_mode !== "off" || v.champion_phase) &&
      tForm("summary.seconds", { seconds: v.step_seconds }),
  );
  const championsText = [
    bans > 0 && tRules("champions.bans", { count: bans }),
    tRules(`champions.repeat.${repeat}`),
  ];

  const rows: { key: string; label: ReactNode; value: string }[] = [
    {
      key: "bracket",
      label: tip("bracket", tRules(`bracketRule.${bracket}`)),
      value: t(`bracket.${bracket}`),
    },
    {
      key: "series",
      label: t("labels.series"),
      value: t(`series.${stage.bracket}`, {
        bestOf: stage.best_of,
        finals: stage.finals_best_of,
      }),
    },
    {
      key: "slots",
      label: t("labels.slots"),
      value: String(tournament.max_participants),
    },
    {
      key: "champions",
      // Sem ban e sem regra, "Podem repetir" não tem o que explicar.
      label:
        bans > 0 || repeat !== "off"
          ? tip("champions", championsText.filter(Boolean).join(" "))
          : t("labels.champions"),
      value: value(
        bans > 0 && t("bans", { count: bans }),
        t(`noRepeat.${repeat}`),
      ),
    },
    { key: "veto", label: t("labels.veto"), value: vetoText },
    {
      key: "checkIn",
      label: tip(
        "checkIn",
        t(checkIn == null ? "tips.noCheckInText" : "tips.checkInText"),
      ),
      value: value(
        checkIn == null
          ? t("noCheckIn")
          : t("checkIn", { window: formatMinutes(checkIn) }),
      ),
    },
    {
      key: "contest",
      label: tip("contest", t("tips.contestText")),
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
