"use client";

import { Add } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { ErrorText } from "@/shared/ui/error-text";
import { Spinner } from "@/shared/ui/spinner";
import type { FormationBoardState } from "../hooks/use-formation-board";
import { useFormationDrop } from "../hooks/use-formation-drop";
import { EmptySlot } from "./empty-slot";
import { FormationBlock } from "./formation-block";
import styles from "./new-team-zone.module.scss";

type Props = { board: FormationBoardState };

/**
 * A área tracejada no fim da grade dos times, do tamanho de um time: soltar
 * um jogador aqui cria um time novo com ele como capitão.
 */
export function NewTeamZone({ board }: Props) {
  const t = useTranslations("Team");
  const { busy, error, ...drop } = useFormationDrop(board, "new");
  return (
    <FormationBlock dashed className={styles.zone} {...drop}>
      <div className={styles.sizer} aria-hidden>
        <div className={styles.sizerHeader} />
        <ul>
          {Array.from({ length: board.formation.team_size }, (_, slot) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: as vagas são iguais
            <EmptySlot key={slot} />
          ))}
        </ul>
      </div>
      <div className={styles.center}>
        <Add size={24} aria-hidden />
        <p>{t("teams.newTeam")}</p>
        {busy && <Spinner label={t("saving")} />}
        {error && <ErrorText>{error}</ErrorText>}
      </div>
    </FormationBlock>
  );
}
