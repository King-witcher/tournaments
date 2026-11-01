import { Locked } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import styles from "./champion-figure.module.scss";

type Props = {
  champion: components["schemas"]["ChampionRef"] | null | undefined;
  /** Sem campeão ainda: o texto no lugar do nome ("a escolher"). */
  empty?: string;
  /** Banido: em cinza, com a borda vermelha e o cadeado. */
  banned?: boolean;
  /** Sorteado pelo fim do tempo. */
  drawn?: boolean;
  /** `sm`: bans e linhas compactas; `md`: o resumo do draft; `lg`: o cartão da partida (quadrado). */
  size?: "sm" | "md" | "lg";
};

/**
 * Um campeão no resumo do draft e nos cartões: o retrato e, embaixo, o nome
 * (ou o que falta); o lugar de quem ainda não escolheu fica com o contorno
 * tracejado, do mesmo tamanho.
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
    <span className={cn(styles.figure, styles[size], banned && styles.banned)}>
      <span className={styles.frame}>
        {champion ? (
          <ChampionPortrait
            slug={champion.slug}
            shape={size === "lg" ? "big" : "roster"}
            out={banned}
          />
        ) : (
          <span aria-hidden className={styles.placeholder} />
        )}
        {banned && (
          <span className={styles.icon}>
            <Locked aria-hidden size={16} />
          </span>
        )}
      </span>
      <span className={cn(styles.name, !champion && styles.empty)}>
        {champion?.name ?? empty}
      </span>
      {drawn && <span className={styles.note}>{t("drawn")}</span>}
    </span>
  );
}
