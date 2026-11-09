import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { fragsText } from "../lib/score";
import styles from "./game-contest-notice.module.scss";

type Props = {
  game: components["schemas"]["GameDetail"];
  names: { a: string; b: string };
  /** O nome de quem contestou. */
  contester: string;
};

/**
 * A contestação de uma partida: quem contestou no título, embaixo o comentário
 * entre aspas e, se veio, o resultado proposto. O título e o texto do `Callout`
 * ficam lado a lado quando cabem; o texto vai como filho para descer de linha.
 */
export function GameContestNotice({ game: g, names, contester }: Props) {
  const t = useTranslations("Match.games");
  const proposal = g.contest_proposal;
  return (
    <Notice kind="warning" title={t("contestedTitle", { name: contester })}>
      <div className={styles.text}>
        <p>{t("contestComment", { comment: g.contest_comment ?? "" })}</p>
        {proposal && (
          <p>
            {t("contestProposal", {
              map: proposal.map.name,
              a: names.a,
              aFrags: fragsText(proposal.a_frags),
              bFrags: fragsText(proposal.b_frags),
              b: names.b,
            })}
          </p>
        )}
      </div>
    </Notice>
  );
}
