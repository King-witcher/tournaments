import { Stack, Tile } from "@carbon/react";
import { useTranslations } from "next-intl";
import { Children, type ReactNode } from "react";

import { PushSuggestion } from "@/features/push/components/push-suggestion";
import { MY_PARTICIPATION_ID } from "../lib/focus";
import styles from "./participation-tile.module.scss";

type Props = {
  /** A etiqueta da situação (`StatusTag`), ao lado do título. */
  tag?: ReactNode;
  /** A ação que falta, a data ou o link da próxima partida (a situação já está na etiqueta). */
  children?: ReactNode;
  /** Sugere ativar as notificações num bloco à parte, abaixo do bloco. */
  suggestPush?: boolean;
};

/**
 * O bloco "Sua situação", logo abaixo do cabeçalho do torneio. Recebe o foco
 * (`tabIndex -1`) depois de inscrever-se, cancelar ou fazer o check-in, que
 * trocam o botão do cabeçalho no refresh. `suggestPush`: embaixo dele, a
 * sugestão (dispensável) de ativar as notificações.
 */
export function ParticipationTile({ tag, children, suggestPush }: Props) {
  const t = useTranslations("Participants.situation");
  const tile = (
    <Tile id={MY_PARTICIPATION_ID} tabIndex={-1}>
      <div className={styles.content}>
        <div className={styles.heading}>
          <h2 className={styles.title}>{t("title")}</h2>
          {tag}
        </div>
        {Children.toArray(children).length > 0 && (
          <div className={styles.body}>{children}</div>
        )}
      </div>
    </Tile>
  );
  if (!suggestPush) return tile;
  return (
    <Stack gap={3}>
      {tile}
      <PushSuggestion />
    </Stack>
  );
}
