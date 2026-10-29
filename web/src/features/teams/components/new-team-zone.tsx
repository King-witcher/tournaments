"use client";

import { useTranslations } from "next-intl";

import { ErrorText } from "@/shared/ui/error-text";
import { Spinner } from "@/shared/ui/spinner";
import type { FormationBoardState } from "../hooks/use-formation-board";
import { FormationBlock } from "./formation-block";
import styles from "./new-team-zone.module.scss";

type Props = { board: FormationBoardState };

/**
 * A área tracejada no fim da grade dos times: soltar um jogador aqui cria um
 * time novo com ele como capitão. Só no computador (no celular o arrastar
 * não funciona; o "Mover para…" do ⋯ tem "Novo time").
 */
export function NewTeamZone({ board }: Props) {
  const t = useTranslations("Team");
  const { busy, error, ...drop } = board.dropProps("new");
  return (
    <FormationBlock dashed className={styles.zone} {...drop}>
      <p className={styles.text}>{t("teams.newTeam")}</p>
      {busy && <Spinner label={t("saving")} />}
      {error && <ErrorText>{error}</ErrorText>}
    </FormationBlock>
  );
}
