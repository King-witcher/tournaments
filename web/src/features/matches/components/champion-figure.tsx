import { useTranslations } from "next-intl";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import styles from "./champion-figure.module.scss";

type Props = {
  champion: components["schemas"]["ChampionRef"] | null | undefined;
  /** Sem campeão ainda: o texto embaixo do lugar dele ("a escolher"). */
  empty?: string;
  /** Banido: em cinza (a linha "Bans" diz o porquê). */
  banned?: boolean;
  /** Sorteado pelo fim do tempo. */
  drawn?: boolean;
  /** `sm`: bans e linhas compactas; `md`: o resumo do draft; `lg`: o cartão da partida (quadrado). */
  size?: "sm" | "md" | "lg";
};

/**
 * Um campeão no resumo do draft e nos cartões: só o retrato (o nome fica para
 * o leitor de tela). O lugar de quem ainda não escolheu fica com o contorno tracejado, do
 * mesmo tamanho, e embaixo o que falta.
 */
export function ChampionFigure({
  champion,
  empty,
  banned = false,
  drawn = false,
  size = "md",
}: Props) {
  const t = useTranslations("Match.veto.draft");
  return (
    <span className={cn(styles.figure, styles[size])}>
      <span className={styles.frame}>
        {champion ? (
          <ChampionPortrait
            slug={champion.slug}
            name={champion.name}
            shape={size === "lg" ? "big" : "roster"}
            out={banned}
          />
        ) : (
          <span aria-hidden className={styles.placeholder} />
        )}
      </span>
      {!champion && <span className={styles.empty}>{empty}</span>}
      {drawn && <span className={styles.note}>{t("drawn")}</span>}
    </span>
  );
}
