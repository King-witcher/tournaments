"use client";

import { useTranslations } from "next-intl";
import { useCallback } from "react";

import { useVetoBoard } from "../hooks/use-veto-board";
import {
  nextMapPickGame,
  type PoolMapState,
  poolStates,
  type TileState,
} from "../lib/veto";
import styles from "./veto-map-board.module.scss";
import { VetoTile } from "./veto-tile";

const TILE: Record<PoolMapState["kind"], TileState> = {
  available: "available",
  banned: "banned",
  picked: "picked",
  decider: "picked",
  kept: "picked",
};

/**
 * O veto de mapas (modo veto): o pool em cartões com a imagem, o banido e o
 * escolhido marcados (com "Partida N" no escolhido, no decisivo e no que sobrou
 * na série par; quem escolheu ou baniu fica para o leitor de tela) e, na vez de quem vê, os que
 * sobraram como botões de banir ou escolher.
 */
export function VetoMapBoard() {
  const t = useTranslations("Match.veto.maps");
  const tGame = useTranslations("Match.games");
  const { veto, bestOf, names, pending, actions } = useVetoBoard();
  const { step } = actions;
  const choose = useCallback((map: string) => step({ map }), [step]);
  if (!veto) return null;
  const states = poolStates(veto, bestOf);
  const kind: "pick" | "ban" = veto.turn?.action === "pick" ? "pick" : "ban";

  const describe = (s: PoolMapState) => {
    switch (s.kind) {
      case "banned":
        return t("banned", { name: names[s.side] });
      case "picked":
        return t("picked", { game: s.game, name: names[s.side] });
      case "decider":
        return t("decider", { game: s.game });
      case "kept":
        return t("kept", { game: s.game });
      case "available":
        return undefined;
    }
  };

  return (
    <div className={styles.board}>
      <ul className={styles.maps}>
        {veto.pool.map((m) => {
          const state = states[m.slug];
          const option = veto.options.includes(m.slug);
          return (
            <VetoTile
              key={m.slug}
              slug={m.slug}
              name={m.name}
              state={TILE[state.kind]}
              note={
                "game" in state
                  ? tGame("game", { number: state.game })
                  : undefined
              }
              described={describe(state)}
              {...(option && {
                action: kind,
                actionText: t(kind),
                actionLabel:
                  kind === "pick"
                    ? t("pickMap", {
                        map: m.name,
                        game: nextMapPickGame(veto),
                      })
                    : t("banMap", { map: m.name }),
                disabled: pending != null,
                onAction: choose,
              })}
            />
          );
        })}
      </ul>
    </div>
  );
}
