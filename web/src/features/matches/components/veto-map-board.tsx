"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import { type PoolMapState, poolStates } from "../lib/veto";
import styles from "./veto-board.module.scss";
import { VetoMapTile } from "./veto-map-tile";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  veto: Schemas["VetoView"];
  bestOf: number;
  names: { a: string; b: string };
};

/**
 * O veto de mapas (modo veto): o pool, em cartões com a imagem do mapa, com
 * o que aconteceu com ele (banido por quem, escolhido por quem para qual jogo, o
 * decisivo) e, na vez de quem vê, o botão de banir ou escolher nos que
 * sobraram.
 */
export function VetoMapBoard({ target, veto, bestOf, names }: Props) {
  const t = useTranslations("Match.veto.maps");
  const action = useVetoAction(target);
  const states = poolStates(veto, bestOf);
  const mapTurn = veto.turn?.target === "map" ? veto.turn : null;
  const label = mapTurn?.action === "pick" ? "pick" : "ban";

  const describe = (s: PoolMapState) => {
    switch (s.kind) {
      case "banned":
        return t("banned", { name: names[s.side] });
      case "picked":
        return t("picked", { game: s.game, name: names[s.side] });
      case "decider":
        return t("decider", { game: s.game });
      case "available":
        return t("available");
    }
  };

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title")}</h3>
      <ul className={styles.maps}>
        {veto.pool.map((m) => {
          const state = states[m.slug];
          return (
            <VetoMapTile
              key={m.slug}
              map={m}
              state={describe(state)}
              out={state.kind === "banned"}
            >
              {veto.options.includes(m.slug) && (
                <Button
                  size="sm"
                  kind={label === "pick" ? "primary" : "danger--tertiary"}
                  pending={action.pending}
                  onClick={() => action.step({ map: m.slug })}
                  aria-label={t(`${label}Map`, { map: m.name })}
                >
                  {t(label)}
                </Button>
              )}
            </VetoMapTile>
          );
        })}
      </ul>
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
