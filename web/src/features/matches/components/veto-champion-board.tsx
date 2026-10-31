"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { Notice } from "@/shared/ui/notice";
import { RadioGroup } from "@/shared/ui/radio-group";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import styles from "./veto-board.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
  /** A regra de não repetição do torneio, para dizer o que ela tira. */
  rule: Schemas["NoRepeatRule"] | null;
};

/**
 * O draft de campeões para quem age agora: no ban, um botão por campeão que
 * o lado dele pode banir (vale para a série inteira); no pick do duelo, um
 * por campeão que ele pode escolher para a partida da vez; na janela do
 * TDM, a partida que ele escolhe (as que faltam) e os campeões dela. Os que
 * não dá para escolher aparecem com o motivo, e um aviso diz quando uma
 * regra cedeu para ele ter campeão. Sem nada para quem vê fazer, não
 * aparece: o que já foi escolhido está em "Partidas da série".
 */
export function VetoChampionBoard({ target, veto, names, rule }: Props) {
  const t = useTranslations("Match.veto.champions");
  const action = useVetoAction(target);
  const turn = veto.turn?.target === "champion" ? veto.turn : null;
  const slots = veto.picks;
  const [chosen, setChosen] = useState<number | null>(null);
  const banning = turn?.action === "ban" && veto.options.length > 0;
  if (!banning && slots.length === 0) return null;
  // A partida da janela: a escolhida, se ainda falta; senão, a primeira.
  const slot = slots.find((s) => s.game === chosen) ?? slots[0];
  const teamPick = turn?.action === "team_pick";
  const options = banning ? veto.options : (slot?.options ?? []);
  const unavailable = new Map(
    (slot?.unavailable ?? []).map((u) => [u.slug, u.reason]),
  );

  const reason = (c: Schemas["VetoChampion"]) => {
    if (c.banned_by) return t("reasons.banned", { name: names[c.banned_by] });
    if (banning) return null;
    switch (unavailable.get(c.slug)) {
      case "teammate":
        return t("reasons.teammate");
      case "opponent":
        return t("reasons.opponent");
      case "repeat":
        return rule
          ? t(`reasons.repeat.${rule.scope}`)
          : t("reasons.repeat.series");
      default:
        return null;
    }
  };

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>
        {banning
          ? t("banTitle")
          : teamPick
            ? t("teamPickTitle")
            : t("pickTitle", { game: slot?.game ?? 1 })}
      </h3>
      {banning && <p className={styles.reason}>{t("banHint")}</p>}
      {teamPick && slots.length > 1 && slot && (
        <RadioGroup
          legendText={t("forGame")}
          orientation="horizontal"
          value={String(slot.game)}
          options={slots.map((s) => ({
            value: String(s.game),
            label: t("game", { game: s.game }),
          }))}
          onChange={(g) => setChosen(Number(g))}
        />
      )}
      {!banning && slot?.gave_way && (
        <Notice kind="warning">{t(`gaveWay.${slot.gave_way}`)}</Notice>
      )}
      <ul className={styles.grid}>
        {veto.champions.map((c) => {
          const why = reason(c);
          return (
            <li key={c.slug} className={styles.champion}>
              {options.includes(c.slug) ? (
                <Button
                  size="sm"
                  kind={banning ? "danger--tertiary" : "primary"}
                  pending={action.pending}
                  onClick={() =>
                    action.step(
                      teamPick && slot
                        ? { champion: c.slug, game: slot.game }
                        : { champion: c.slug },
                    )
                  }
                  aria-label={
                    banning
                      ? t("banChampion", { champion: c.name })
                      : t("pickChampion", {
                          champion: c.name,
                          game: slot?.game ?? 1,
                        })
                  }
                >
                  {c.name}
                </Button>
              ) : (
                <span className={styles.unavailable}>{c.name}</span>
              )}
              {why && <span className={styles.reason}>{why}</span>}
            </li>
          );
        })}
      </ul>
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
