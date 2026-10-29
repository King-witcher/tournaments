import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { Section } from "@/shared/ui/section";
import type { MatchTarget } from "../lib/match-target";
import { GameRow } from "./game-row";
import styles from "./match-games-panel.module.scss";
import { NextGameRow } from "./next-game-row";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  maps: Schemas["Map"][];
  contestMinutes: number;
};

/**
 * Os mapas da série, uma linha por mapa (nº, mapa, resultado, frags, estado
 * e o que dá para fazer nele); por último, a linha do próximo mapa, enquanto
 * a série aceita. Some com a partida que ainda não tem lados.
 */
export function MatchGamesPanel({
  target,
  detail,
  maps,
  contestMinutes,
}: Props) {
  const t = useTranslations("Match.games");
  const formatMinutes = useFormatMinutes();
  const s = detail.summary;
  const names = { a: detail.a?.nickname ?? "", b: detail.b?.nickname ?? "" };
  const next = detail.next_game;
  const open = s.state === "ready";
  if (!open && detail.games.length === 0) return null;

  return (
    <Section
      title={t("title")}
      description={
        open ? t("hint", { window: formatMinutes(contestMinutes) }) : undefined
      }
    >
      <div>
        {/* Os nomes das colunas, para correr o olho; cada linha também se
            explica sozinha (o leitor de tela lê "Mapa 1", "Frags"). */}
        <div aria-hidden className={styles.header}>
          <span className={styles.num}>{t("columns.number")}</span>
          <span className={styles.map}>{t("columns.map")}</span>
          <span className={styles.result}>{t("columns.result")}</span>
          <span className={styles.frags}>{t("columns.frags")}</span>
          <span className={styles.status}>{t("columns.status")}</span>
        </div>
        <ol aria-label={t("title")}>
          {detail.games.map((game) => (
            <GameRow
              key={game.number}
              target={target}
              game={game}
              names={names}
              maps={maps}
              canManage={detail.can_manage}
              last={game.number === detail.games.length}
            />
          ))}
          {next != null && (
            <NextGameRow
              target={target}
              number={next}
              names={names}
              maps={maps}
              plays={detail.viewer_side != null}
              enters={detail.can_manage && detail.viewer_side == null}
              veto={
                detail.veto && {
                  reportable: detail.veto.reportable,
                  map:
                    detail.veto.games.find((g) => g.game === next)?.map ?? null,
                }
              }
            />
          )}
        </ol>
      </div>
    </Section>
  );
}
