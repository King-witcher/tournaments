"use client";

import { useTranslations } from "next-intl";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
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
  const tMaps = useTranslations("Match.veto.maps");
  const { veto, names, pending, actions } = useVetoBoard();
  const text = useChampionTileText(names, rule);
  if (!veto) return null;
  const turn = veto.turn?.target === "champion" ? veto.turn : null;
  const banning = turn?.action === "ban";
  const pickSlot = banning ? undefined : slot;
  const options = banning ? veto.options : (pickSlot?.options ?? []);
  const tiles = championTiles(veto, pickSlot);
  const game = pickSlot?.game ?? turn?.pick_for ?? 1;
  const { hint, described } = text;

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
              image={
                <ChampionPortrait slug={c.slug} out={tile.state === "banned"} />
              }
              name={c.name}
              state={tile.state}
              portrait
              hint={hint(tile)}
              described={described(tile)}
              action={
                options.includes(c.slug)
                  ? {
                      kind: banning ? "ban" : "pick",
                      text: banning ? tMaps("ban") : tMaps("pick"),
                      label: banning
                        ? t("banChampion", { champion: c.name })
                        : t("pickChampion", { champion: c.name, game }),
                      disabled: pending != null,
                      onClick: () =>
                        actions.step(
                          turn?.action === "team_pick"
                            ? { champion: c.slug, game }
                            : { champion: c.slug },
                        ),
                    }
                  : undefined
              }
            />
          );
        })}
      </ul>
    </div>
  );
}
