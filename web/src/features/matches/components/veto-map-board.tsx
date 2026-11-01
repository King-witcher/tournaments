"use client";

import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import { ErrorText } from "@/shared/ui/error-text";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import { type PoolMapState, poolStates, type TileState } from "../lib/veto";
import styles from "./veto-board.module.scss";
import { VetoTile } from "./veto-tile";

const TILE: Record<PoolMapState["kind"], TileState> = {
  available: "available",
  banned: "banned",
  picked: "picked",
  decider: "picked",
};

/**
 * O veto de mapas (modo veto): o pool em cartões com a imagem, o banido e o
 * escolhido marcados (com "Partida N" no escolhido e no decisivo; quem
 * escolheu ou baniu fica para o leitor de tela) e, na vez de quem vê, os que
 * sobraram como botões de banir ou escolher.
 */
export function VetoMapBoard({ target }: { target: MatchTarget }) {
  const t = useTranslations("Match.veto.maps");
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const action = useVetoAction(target);
  const veto = detail.veto;
  if (!veto) return null;
  const states = poolStates(veto, detail.summary.best_of);
  const kind = veto.turn?.action === "pick" ? "pick" : "ban";

  const describe = (s: PoolMapState) => {
    switch (s.kind) {
      case "banned":
        return t("banned", { name: names[s.side] });
      case "picked":
        return t("picked", { game: s.game, name: names[s.side] });
      case "decider":
        return t("decider", { game: s.game });
      case "available":
        return undefined;
    }
  };

  return (
    <div className={styles.board}>
      <ul className={styles.maps}>
        {veto.pool.map((m) => {
          const state = states[m.slug];
          return (
            <VetoTile
              key={m.slug}
              image={
                <MapThumbnail slug={m.slug} out={state.kind === "banned"} />
              }
              name={m.name}
              state={TILE[state.kind]}
              note={
                "game" in state ? t("game", { game: state.game }) : undefined
              }
              described={describe(state)}
              action={
                veto.options.includes(m.slug)
                  ? {
                      kind,
                      text: t(kind),
                      label: t(`${kind}Map`, { map: m.name }),
                      onClick: () => action.step({ map: m.slug }),
                    }
                  : undefined
              }
            />
          );
        })}
      </ul>
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
