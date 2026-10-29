"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import styles from "./veto-board.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  veto: Schemas["VetoView"];
  game: number;
  names: { a: string; b: string };
  /** O lado de quem vê, se joga. */
  mine: Schemas["Side"] | null;
  /** A regra de não repetição do torneio, para explicar o que ela tira. */
  rule: Schemas["NoRepeatScope"] | null;
};

/**
 * Os campeões do mapa em andamento: na vez de quem vê, um botão por
 * campeão que ele pode banir ou escolher; os outros aparecem com o motivo
 * (banido neste mapa e por quem; já usado, pela regra de não repetição, por
 * quem vê ou pelo adversário).
 */
export function VetoChampionBoard(props: Props) {
  const { target, veto, game, names, mine, rule } = props;
  const t = useTranslations("Match.veto.champions");
  const action = useVetoAction(target);
  const turn = veto.turn?.target === "champion" ? veto.turn : null;
  const label = turn?.action === "pick" ? "pick" : "ban";
  const myTurn = turn != null && mine != null && turn.sides.includes(mine);

  const reasons = (c: Schemas["VetoChampion"]) => {
    const list: string[] = [];
    if (c.banned_by) list.push(t("banned", { name: names[c.banned_by] }));
    for (const side of ["a", "b"] as const) {
      if (c[`used_by_${side}`] && rule) {
        list.push(
          side === mine
            ? t(`usedByYou.${rule}`)
            : t(`usedBy.${rule}`, { name: names[side] }),
        );
      }
    }
    return list;
  };

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title", { game })}</h3>
      {myTurn && <p className={styles.reason}>{t(`${label}Hint`)}</p>}
      <ul className={styles.grid}>
        {veto.champions.map((c) => {
          const why = reasons(c);
          return (
            <li key={c.slug} className={styles.champion}>
              {myTurn && veto.options.includes(c.slug) ? (
                <Button
                  size="sm"
                  kind={label === "pick" ? "primary" : "danger--tertiary"}
                  pending={action.pending}
                  onClick={() => action.step({ champion: c.slug })}
                  aria-label={t(`${label}Champion`, { champion: c.name })}
                >
                  {c.name}
                </Button>
              ) : (
                <span className={styles.unavailable}>{c.name}</span>
              )}
              {why.length > 0 && (
                <span className={styles.reason}>{why.join(" · ")}</span>
              )}
            </li>
          );
        })}
      </ul>
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
