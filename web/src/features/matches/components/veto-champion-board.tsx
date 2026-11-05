"use client";

import { useTranslations } from "next-intl";
import { useCallback } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { useChampionTileText } from "../hooks/use-champion-tile-text";
import { useVetoBoard } from "../hooks/use-veto-board";
import { championTiles } from "../lib/champion-tiles";
import styles from "./veto-champion-board.module.scss";
import { VetoTile } from "./veto-tile";

type Schemas = components["schemas"];

type Props = {
  /** A regra de não repetição do torneio, para dizer o que ela tira. */
  rule: Schemas["NoRepeatRule"] | null;
  /** O pick de quem vê agora (no pick por jogador, a partida da vez ou a escolhida). */
  slot: Schemas["VetoPickSlot"] | undefined;
};

/**
 * A grade do draft: os retratos lado a lado, cada um só com a resposta a "posso
 * escolher agora?": o banido com a marca, o que a regra tira do pick de quem
 * vê apagado, com o motivo numa dica, e, na vez dele, os que pode banir ou
 * escolher como botões. Quem escolheu o quê fica no resumo do draft. Um aviso
 * diz quando uma regra cedeu para ele ter campeão.
 */
export function VetoChampionBoard({ rule, slot }: Props) {
  const t = useTranslations("Match.veto.champions");
  const tMember = useTranslations("Member");
  const tMaps = useTranslations("Match.veto.maps");
  const { veto, names, pending, actions } = useVetoBoard();
  const text = useChampionTileText(names, rule);
  const turn = veto?.turn?.target === "champion" ? veto.turn : null;
  const banning = turn?.action === "ban";
  const pickSlot = banning ? undefined : slot;
  const game = pickSlot?.game ?? turn?.pick_for ?? 1;
  const teamPick = turn?.action === "team_pick";
  const player = pickSlot?.player ?? undefined;
  const { step } = actions;
  const choose = useCallback(
    (champion: string) =>
      step(teamPick ? { champion, game, player } : { champion }),
    [step, teamPick, game, player],
  );
  if (!veto) return null;
  const options = banning ? veto.options : (pickSlot?.options ?? []);
  const tiles = championTiles(veto, pickSlot);
  const { hint, described } = text;
  // O capitão escolhendo por um colega: é o que diz para quem é o campeão.
  const forTeam = veto.can_act.pick_for_team && pickSlot?.player != null;
  const pickLabel = (champion: string) =>
    forTeam && !veto.nameless
      ? t("pickChampionFor", {
          champion,
          game,
          player: shownNickname(
            pickSlot?.nickname ?? "",
            tMember("deletedAccount"),
          ),
        })
      : t("pickChampion", { champion, game });

  return (
    <div className={styles.board}>
      {pickSlot?.gave_way && (
        <Notice kind="warning">{t(`gaveWay.${pickSlot.gave_way}`)}</Notice>
      )}
      <ul className={styles.champions} aria-label={t("grid")}>
        {veto.champions.map((c) => {
          const tile = tiles[c.slug];
          return (
            <VetoTile
              key={c.slug}
              slug={c.slug}
              name={c.name}
              state={tile.state}
              portrait
              hint={hint(tile)}
              described={described(tile)}
              {...(options.includes(c.slug) && {
                action: banning ? ("ban" as const) : ("pick" as const),
                actionText: banning ? tMaps("ban") : tMaps("pick"),
                actionLabel: banning
                  ? t("banChampion", { champion: c.name })
                  : pickLabel(c.name),
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
