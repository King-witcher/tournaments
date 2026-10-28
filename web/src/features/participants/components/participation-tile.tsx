import { Tile } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { MY_PARTICIPATION_ID } from "../lib/focus";
import styles from "./participation-tile.module.scss";

type Props = {
  /** A etiqueta da situação (`StatusTag`), ao lado do título. */
  tag?: ReactNode;
  /** As frases da situação e, se houver, o link da próxima partida. */
  children?: ReactNode;
};

/**
 * O bloco "Sua situação", logo abaixo do cabeçalho do torneio. Recebe o foco
 * (`tabIndex -1`) depois de inscrever-se, cancelar ou fazer o check-in, que
 * trocam o botão do cabeçalho no refresh.
 */
export function ParticipationTile({ tag, children }: Props) {
  const t = useTranslations("Participants.situation");
  return (
    <Tile id={MY_PARTICIPATION_ID} tabIndex={-1}>
      <div className={styles.content}>
        <div className={styles.heading}>
          <h2 className={styles.title}>{t("title")}</h2>
          {tag}
        </div>
        {children && <div className={styles.body}>{children}</div>}
      </div>
    </Tile>
  );
}
