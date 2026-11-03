"use client";

import { useTranslations } from "next-intl";

import { StatusTag } from "@/shared/ui/status-tag";
import { useVetoBoard } from "../hooks/use-veto-board";
import { ChampionFigure } from "./champion-figure";
import { DraftGameRow } from "./draft-game-row";
import styles from "./draft-summary.module.scss";

/**
 * O draft da série pela proximidade, sem relatório de texto: uma coluna por
 * lado sob o próprio nome (o de quem vê com "Você") e, entre elas, os bans e
 * cada partida (o mapa no meio, o campeão de cada lado ao lado dele).
 * `current` destaca a partida da vez.
 */
export function DraftSummary({ current }: { current?: number }) {
  const t = useTranslations("Match.veto.draft");
  const { veto, mine, names, viewerId, hideEmptyMaps } = useVetoBoard();
  if (!veto) return null;
  const head = (side: "a" | "b") => (
    <th scope="col">
      <div className={styles.side}>
        {names[side]}
        {mine === side && (
          <StatusTag status="neutral" size="sm">
            {t("you")}
          </StatusTag>
        )}
      </div>
    </th>
  );
  const bans = (side: "a" | "b") => (
    <ul className={styles.bans}>
      {veto.bans
        .filter((b) => b.side === side)
        .map((b) => (
          <li key={b.champion.slug}>
            <ChampionFigure
              champion={b.champion}
              banned
              drawn={b.timeout}
              size="sm"
            />
          </li>
        ))}
    </ul>
  );

  return (
    <table className={styles.table}>
      <caption className={styles.srOnly}>{t("caption")}</caption>
      <thead>
        <tr>
          {head("a")}
          <th scope="col">
            <span className={styles.srOnly}>{t("gameColumn")}</span>
          </th>
          {head("b")}
        </tr>
      </thead>
      <tbody>
        {veto.bans.length > 0 && (
          <tr>
            <td>{bans("a")}</td>
            <th scope="row" className={styles.middle}>
              {t("bans")}
            </th>
            <td>{bans("b")}</td>
          </tr>
        )}
        {veto.games.map((g) => (
          <DraftGameRow
            key={g.game}
            veto={veto}
            game={g}
            viewerId={viewerId}
            current={g.game === current}
            hideEmptyMap={hideEmptyMaps}
          />
        ))}
      </tbody>
    </table>
  );
}
