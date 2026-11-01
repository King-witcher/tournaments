import { Column, Grid } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Section } from "@/shared/ui/section";
import { gameRowId } from "../lib/match-target";
import styles from "./game-step.module.scss";

type Props = {
  number: number;
  /** Ao lado do título, à direita (o desfazer do veto, na partida da vez). */
  action?: ReactNode;
  children: ReactNode;
};

/**
 * O passo de uma partida: o título "Partida N" (o estado já está embaixo
 * do nome no indicador e na frase do cartão) e o conteúdo (o veto da partida e o cartão), em 12 das 16 colunas no lg e 10
 * no xlg, para os três blocos do cartão não ficarem longe um do outro numa
 * tela larga.
 */
export function GameStep({ number, action, children }: Props) {
  const t = useTranslations("Match.games");
  return (
    // tabIndex -1: recebe o foco depois de agir na partida (o botão usado
    // some) e de anulá-la.
    <div id={gameRowId(number)} tabIndex={-1} className={styles.step}>
      <Grid>
        <Column sm={4} md={8} lg={12} xlg={10}>
          <Section title={t("game", { number })} action={action}>
            {children}
          </Section>
        </Column>
      </Grid>
    </div>
  );
}
