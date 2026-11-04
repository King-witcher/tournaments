import { Incomplete } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import type { Href } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Link } from "@/shared/ui/link";
import { crossResult } from "../lib/crosstable";
import { fragsText } from "../lib/score";
import styles from "./crosstable-cell.module.scss";

type Props = {
  match: components["schemas"]["BracketMatch"];
  /** O time da linha: a célula é lida do ponto de vista dele. */
  team: number;
  /** Os nomes da linha e da coluna, para o leitor de tela e a dica. */
  row: string;
  column: string;
  href: Href;
};

/** Os frags levam espaço: colado, o traço some ao lado do sinal de menos. */
function scoreText(own: number, other: number, frags = false) {
  return frags ? `${fragsText(own)} – ${fragsText(other)}` : `${own}–${other}`;
}

/**
 * Uma célula da tabela cruzada, que inteira é o link da série: o resultado de
 * quem está na linha. Vitória e derrota se distinguem pela letra e pelo peso,
 * não só pela cor; a série em andamento, pelo ícone e pelo itálico, como o
 * placar provisório do cartão; a pendente é um traço na cor de link, sem
 * resultado, para ainda parecer clicável no celular, sem hover.
 */
export function CrosstableCell({ match, team, row, column, href }: Props) {
  const t = useTranslations("Bracket");
  const tSaid = useTranslations("Bracket.crosstable.described");
  const result = crossResult(match, team);

  let won = false;
  let said = tSaid("pending");
  let content = <span aria-hidden>—</span>;
  if (result.kind === "playing") {
    const score = scoreText(result.own, result.other);
    said = tSaid("playing", { score });
    content = (
      <>
        <Incomplete />
        {score}
      </>
    );
  } else if (result.kind === "walkover" || result.kind === "score") {
    won = result.won;
    const mark = t(won ? "crosstable.won" : "crosstable.lost");
    if (result.kind === "walkover") {
      said = tSaid(won ? "wonWalkover" : "lostWalkover");
      content = (
        <>
          <span>{mark}</span>
          {t("walkover")}
        </>
      );
    } else {
      const score = scoreText(result.own, result.other, result.frags);
      said = tSaid(won ? "won" : "lost", {
        score: result.frags ? tSaid("frags", { score }) : score,
      });
      content = (
        <>
          <span>{mark}</span>
          {score}
        </>
      );
    }
  }
  const label = `${t("matchSides", { a: row, b: column })}, ${said}`;

  return (
    <Link
      href={href}
      aria-label={label}
      title={label}
      className={cn(
        styles.cell,
        won && styles.won,
        result.kind === "playing" && styles.playing,
      )}
    >
      {content}
    </Link>
  );
}
