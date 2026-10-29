import { Stack } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";
import { formatPoints } from "../lib/format-points";
import styles from "./rating-summary.module.scss";
import { RatingValue } from "./rating-value";

type Props = {
  standing: components["schemas"]["RankingEntry"];
  /** Quantos estão no ranking (sem ele, a posição sai sozinha). */
  total?: number;
  /** Soma dos ajustes, já dentro do rating. */
  adjusted: number;
  provisionalGames: number;
};

/**
 * O resumo do rating de alguém, em pares rótulo e valor: rating (com a marca
 * de provisório e a explicação), posição, mapas com vitórias e derrotas, o
 * último mapa e, se houver, a soma dos ajustes da organização.
 */
export function RatingSummary({
  standing: s,
  total,
  adjusted,
  provisionalGames,
}: Props) {
  const t = useTranslations("Ratings.member");
  const locale = useLocale();
  const rows = [
    {
      key: "rating",
      cells: [
        t("rating"),
        <RatingValue
          key="value"
          rating={s.rating}
          provisional={s.provisional}
        />,
      ],
    },
    {
      key: "position",
      cells: [
        t("position"),
        total === undefined
          ? t("positionShort", { position: s.position })
          : t("positionValue", { position: s.position, total }),
      ],
    },
    {
      key: "games",
      cells: [
        t("games"),
        t("gamesValue", { count: s.games, wins: s.wins, losses: s.losses }),
      ],
    },
    {
      key: "lastGame",
      cells: [
        t("lastGame"),
        s.last_game_at ? (
          <LocalDateTime key="value" value={s.last_game_at} />
        ) : (
          t("noGame")
        ),
      ],
    },
    ...(adjusted !== 0
      ? [
          {
            key: "adjusted",
            cells: [
              t("adjusted"),
              t("adjustedValue", { points: formatPoints(adjusted, locale) }),
            ],
          },
        ]
      : []),
  ];

  return (
    <Stack gap={4}>
      <StructuredList label={t("summaryLabel")} rows={rows} condensed />
      {s.provisional && (
        <p className={styles.hint}>
          {t("provisionalHint", { count: provisionalGames })}
        </p>
      )}
    </Stack>
  );
}
