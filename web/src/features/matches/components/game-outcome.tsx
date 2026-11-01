import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { LocalTime } from "@/shared/ui/local-time";
import { otherSide } from "../lib/veto";
import { DeadlineCountdown } from "./deadline-countdown";
import styles from "./game-outcome.module.scss";

type Schemas = components["schemas"];

type Props = {
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  /** Sem o placar alinhado no cartão (partida sem mapa): os frags na frase. */
  inlineFrags: boolean;
};

/**
 * O que a partida enviada diz, embaixo do placar: quem venceu (sem
 * confirmar, como o que quem enviou diz: o negrito ficaria parecendo fato),
 * no TDM quem jogou e, só quando diz algo, o estado: fora do placar até
 * confirmar e até quando confirma sozinha; contestada, por quem e o
 * comentário; confirmada que quem vê ainda contesta.
 */
export function GameOutcome({ game: g, names, inlineFrags }: Props) {
  const t = useTranslations("Match.games");
  const tMember = useTranslations("Member");
  const winner = names[g.winner];
  const settled = g.status === "confirmed";
  const nicks = (list: Schemas["TeamMember"][]) =>
    list
      .map((m) => shownNickname(m.nickname, tMember("deletedAccount")))
      .join(", ");
  const lineup =
    g.lineup_a.length > 1 || g.lineup_b.length > 1
      ? t("lineup", { a: nicks(g.lineup_a), b: nicks(g.lineup_b) })
      : null;
  const deadline = g.confirm_by;
  const status: ReactNode[] = [];
  if (g.status === "pending") {
    status.push(t("notCounted"));
    if (deadline)
      status.push(
        t.rich("pending", {
          at: () => <LocalTime value={deadline} />,
          time: () => <DeadlineCountdown until={deadline} />,
        }),
      );
  } else if (g.status === "contested") {
    if (g.reported_side)
      status.push(
        t("contested", {
          name: g.contested_by ?? names[otherSide(g.reported_side)],
          comment: g.contest_comment ?? "",
        }),
      );
    status.push(t("notCountedContested"));
  } else if (g.confirmed_via && g.can_contest) {
    status.push(t(`confirmed.${g.confirmed_via}`), t("lateContest", names));
  }

  return (
    <div className={styles.outcome}>
      <p className={settled ? styles.settled : undefined}>
        {settled || !g.reported_side
          ? t("won", { name: winner })
          : t("claim", {
              reporter: names[g.reported_side],
              winner,
              // "Fulano enviou que venceu", sem o nome duas vezes.
              self: g.reported_side === g.winner ? "yes" : "no",
            })}
        {inlineFrags && g.a_frags != null && g.b_frags != null && (
          <span className={styles.frags}>
            {" "}
            {t("fragsInline", { a: g.a_frags, b: g.b_frags })}
          </span>
        )}
      </p>
      {lineup && <p className={styles.note}>{lineup}</p>}
      {status.length > 0 && (
        <p className={styles.note}>
          {status.map((part, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: frases fixas, na mesma ordem.
            <span key={i}>
              {i > 0 && " "}
              {part}
            </span>
          ))}
        </p>
      )}
    </div>
  );
}
