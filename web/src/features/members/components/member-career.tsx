import { useLocale, useTranslations } from "next-intl";

import { formatPoints } from "@/features/ratings/lib/format-points";
import type { components } from "@/shared/api/schema";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";
import { usePlacement } from "../hooks/use-placement";

type Props = {
  career: components["schemas"]["MemberCareer"];
};

/**
 * "Carreira" na Visão geral da pessoa, em pares rótulo e valor: séries e
 * partidas V–D, saldo de frags, títulos, pódios, melhor colocação e a
 * sequência atual em séries. Conta todos os torneios da comunidade, com
 * rating ou não. Sem nenhuma partida nem série, só o estado.
 */
export function MemberCareer({ career: c }: Props) {
  const t = useTranslations("Member.career");
  const locale = useLocale();
  const place = usePlacement();
  const series =
    c.series.draws > 0
      ? t("recordDraws", c.series)
      : t("record", { wins: c.series.wins, losses: c.series.losses });
  const played =
    c.series.wins +
    c.series.losses +
    c.series.draws +
    c.games.wins +
    c.games.losses;
  const rows = [
    { key: "series", cells: [t("series"), series] },
    { key: "games", cells: [t("games"), t("record", c.games)] },
    {
      key: "frags",
      cells: [
        t("frags"),
        t("fragsValue", {
          diff: formatPoints(c.frags_for - c.frags_against, locale),
          for: c.frags_for,
          against: c.frags_against,
        }),
      ],
    },
    { key: "titles", cells: [t("titles"), String(c.titles)] },
    { key: "podiums", cells: [t("podiums"), String(c.podiums)] },
    {
      key: "best",
      cells: [
        t("best"),
        c.best_placement ? place(c.best_placement) : t("none"),
      ],
    },
    {
      key: "streak",
      cells: [
        t("streak"),
        c.streak
          ? t(c.streak.won ? "streakWon" : "streakLost", {
              count: c.streak.count,
            })
          : t("none"),
      ],
    },
  ];

  return (
    <Panel>
      <Section title={t("title")} description={t("description")}>
        {played === 0 ? (
          <p>{t("empty")}</p>
        ) : (
          <StructuredList label={t("label")} rows={rows} condensed />
        )}
      </Section>
    </Panel>
  );
}
