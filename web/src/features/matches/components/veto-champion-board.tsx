"use client";

import { useTranslations } from "next-intl";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
import type { components } from "@/shared/api/schema";
import { ErrorText } from "@/shared/ui/error-text";
import { Notice } from "@/shared/ui/notice";
import { useChampionTileText } from "../hooks/use-champion-tile-text";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { useVetoAction } from "../hooks/use-veto-action";
import { championTiles } from "../lib/champion-tiles";
import type { MatchTarget } from "../lib/match-target";
import styles from "./veto-board.module.scss";
import { VetoTile } from "./veto-tile";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  /** A regra de não repetição do torneio, para dizer o que ela tira. */
  rule: Schemas["NoRepeatRule"] | null;
  /** O pick de quem vê agora (na janela do TDM, a partida escolhida). */
  slot: Schemas["VetoPickSlot"] | undefined;
};

/**
 * A grade do draft: os retratos lado a lado com o nome, o banido e o
 * escolhido marcados como no veto de mapas ("Partida N" no escolhido), o que
 * a regra tira do pick de quem vê com o motivo curto e, na vez dele, os que
 * ele pode banir ou escolher como botões. Um aviso diz quando uma regra
 * cedeu para ele ter campeão.
 */
export function VetoChampionBoard({ target, rule, slot }: Props) {
  const t = useTranslations("Match.veto.champions");
  const tMaps = useTranslations("Match.veto.maps");
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const action = useVetoAction(target);
  const text = useChampionTileText(names, rule);
  const veto = detail.veto;
  if (!veto) return null;
  const turn = veto.turn?.target === "champion" ? veto.turn : null;
  const banning = turn?.action === "ban";
  const pickSlot = banning ? undefined : slot;
  const options = banning ? veto.options : (pickSlot?.options ?? []);
  const tiles = championTiles(veto, pickSlot);
  const game = pickSlot?.game ?? turn?.pick_for ?? 1;
  const { note, described } = text;

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
                <ChampionPortrait
                  slug={c.slug}
                  out={tile.state !== "available" && tile.state !== "picked"}
                />
              }
              name={c.name}
              state={tile.state}
              note={note(tile)}
              described={described(tile)}
              action={
                options.includes(c.slug)
                  ? {
                      kind: banning ? "ban" : "pick",
                      text: banning ? tMaps("ban") : tMaps("pick"),
                      label: banning
                        ? t("banChampion", { champion: c.name })
                        : t("pickChampion", { champion: c.name, game }),
                      onClick: () =>
                        action.step(
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
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
