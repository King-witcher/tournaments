import { CheckmarkOutline, CircleDash, Incomplete } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

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
 * quem está na linha. A série encerrada é o check e o placar sobre um fundo teal
 * (vitória) ou roxo (derrota), a paleta divergente do Carbon para desempenho;
 * a em andamento, o parcial sobre cinza, com o meio círculo; a pendente, só o
 * ícone de pendente, sem fundo: os três estados têm o símbolo do cartão. A cor
 * não fala sozinha: o nome do link diz o resultado por extenso.
 */
export function CrosstableCell({ match, team, row, column, href }: Props) {
  const t = useTranslations("Bracket");
  const tSaid = useTranslations("Bracket.crosstable.described");
  const result = crossResult(match, team);

  let tone: string | undefined;
  let said = tSaid("pending");
  let content: ReactNode = <CircleDash />;
  if (result.kind === "playing") {
    const score = scoreText(result.own, result.other);
    said = tSaid("playing", { score });
    tone = styles.playing;
    content = (
      <>
        <Incomplete />
        {score}
      </>
    );
  } else if (result.kind === "walkover" || result.kind === "score") {
    tone = result.won ? styles.won : styles.lost;
    if (result.kind === "walkover") {
      said = tSaid(result.won ? "wonWalkover" : "lostWalkover");
      content = (
        <>
          <CheckmarkOutline />
          {t("walkover")}
        </>
      );
    } else {
      const score = scoreText(result.own, result.other, result.frags);
      said = tSaid(result.won ? "won" : "lost", {
        score: result.frags ? tSaid("frags", { score }) : score,
      });
      content = (
        <>
          <CheckmarkOutline />
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
      className={cn(styles.cell, tone)}
    >
      {content}
    </Link>
  );
}
