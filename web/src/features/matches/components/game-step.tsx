import { Column, Grid } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { gameRowId } from "../lib/match-target";
import { GAME_STATUS } from "../lib/status";
import styles from "./game-step.module.scss";

type Props = {
  number: number;
  /** A partida enviada (a etiqueta do estado ao lado do título). */
  game?: components["schemas"]["GameDetail"];
  /** Ao lado do título, à direita (o desfazer do veto, na partida da vez). */
  action?: ReactNode;
  children: ReactNode;
};

/**
 * O passo de uma partida: o título "Partida N" com a etiqueta do estado e o
 * conteúdo (o veto da partida e o cartão), em 12 das 16 colunas no lg e 10
 * no xlg, para os três blocos do cartão não ficarem longe um do outro numa
 * tela larga.
 */
export function GameStep({ number, game, action, children }: Props) {
  const t = useTranslations("Match.games");
  return (
    // tabIndex -1: recebe o foco depois de agir na partida (o botão usado
    // some) e de anulá-la.
    <div id={gameRowId(number)} tabIndex={-1} className={styles.step}>
      <Grid>
        <Column sm={4} md={8} lg={12} xlg={10}>
          <Section
            title={
              <span className={styles.title}>
                {t("game", { number })}
                {game && (
                  <StatusTag status={GAME_STATUS[game.status]}>
                    {t(`badge.${game.status}`)}
                  </StatusTag>
                )}
              </span>
            }
            action={action}
          >
            {children}
          </Section>
        </Column>
      </Grid>
    </div>
  );
}
