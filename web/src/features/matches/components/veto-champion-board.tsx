"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { Notice } from "@/shared/ui/notice";
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
  rule: Schemas["NoRepeatRule"] | null;
};

/**
 * Os campeões do mapa em andamento: na vez de quem vê (nos times, o ban é
 * do capitão e a escolha simultânea é de cada um), um botão por campeão que
 * ele pode banir ou escolher; os outros aparecem com o motivo (banido neste
 * mapa e por quem; já usado, pela regra de não repetição, por quem vê ou
 * pelo adversário; no TDM, escolhido neste mapa e por quem). No TDM, o que a
 * regra tira de quem vê é o dele (pela regra do jogador, só o que ele mesmo
 * usou), e um aviso diz quando a regra cedeu para ele ter campeão: os que
 * ela tira voltam às opções, com o motivo ao lado.
 */
export function VetoChampionBoard(props: Props) {
  const { target, veto, game, names, mine, rule } = props;
  const t = useTranslations("Match.veto.champions");
  const tMember = useTranslations("Member");
  const action = useVetoAction(target);
  const turn = veto.turn?.target === "champion" ? veto.turn : null;
  const teamPick = turn?.action === "team_pick";
  const teams = veto.team_pick_seconds != null;
  const label = turn?.action === "ban" ? "ban" : "pick";
  // As opções só vêm para quem age agora (o capitão, ou na escolha
  // simultânea quem ainda não escolheu).
  const myTurn = turn != null && veto.options.length > 0;
  const picks = veto.games.find((g) => g.game === game)?.team_picks ?? [];

  const used = (c: Schemas["VetoChampion"], side: Schemas["Side"]) => {
    if (!rule) return null;
    const { scope, who } = rule;
    // Nos times, o lado de quem vê: o que a regra tira dele.
    if (teams && side === mine) {
      if (!c.used_by_you) return null;
      return t(
        who === "team" ? `usedByYourTeam.${scope}` : `usedByYou.${scope}`,
      );
    }
    if (!c[`used_by_${side}`]) return null;
    if (side === mine) return t(`usedByYou.${scope}`);
    // Pela regra do jogador, o lado junta o que ela tira de cada um.
    return teams && who === "player"
      ? t(`usedByPlayerOf.${scope}`, { name: names[side] })
      : t(`usedBy.${scope}`, { name: names[side] });
  };

  const reasons = (c: Schemas["VetoChampion"]) => {
    const list: string[] = [];
    if (c.banned_by) list.push(t("banned", { name: names[c.banned_by] }));
    for (const p of picks.filter((p) => p.champion.slug === c.slug)) {
      const name = shownNickname(p.nickname, tMember("deletedAccount"));
      list.push(t("pickedBy", { name }));
    }
    for (const side of ["a", "b"] as const) {
      const reason = used(c, side);
      if (reason) list.push(reason);
    }
    return list;
  };

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title", { game })}</h3>
      {myTurn && (
        <p className={styles.reason}>
          {t(teamPick ? "teamPickHint" : `${label}Hint`)}
        </p>
      )}
      {myTurn && veto.gave_way && (
        <Notice kind="warning">{t(`gaveWay.${veto.gave_way}`)}</Notice>
      )}
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
