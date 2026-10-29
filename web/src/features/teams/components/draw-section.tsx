"use client";

import { useTranslations } from "next-intl";

import { Section } from "@/shared/ui/section";
import { drawPool, type Formation } from "../lib/formation";
import { DrawOption } from "./draw-option";
import styles from "./draw-section.module.scss";

type Props = {
  tenant: string;
  tournament: string;
  formation: Formation;
  /** Sorteou: a formação nova. */
  onDrawn: (formation: Formation) => void;
};

/**
 * "Sortear" e "Sortear balanceado", cada um com uma frase do que faz. Os dois
 * refazem todos os times com quem pode jogar (com check-in, só quem fez):
 * com times já formados, perguntam antes. Sem gente para um time, ficam
 * desabilitados com o motivo.
 */
export function DrawSection({ tenant, tournament, formation, onDrawn }: Props) {
  const t = useTranslations("Team.draw");
  const pool = drawPool(formation);
  const size = formation.team_size;
  const reason =
    pool >= size
      ? undefined
      : t(formation.has_check_in ? "shortCheckIn" : "short", {
          size,
          count: pool,
        });

  return (
    <Section
      title={t("title")}
      description={formation.has_check_in ? t("whatCheckIn") : t("what")}
    >
      <ul className={styles.options}>
        {(["random", "balanced"] as const).map((method) => (
          <DrawOption
            key={method}
            tenant={tenant}
            tournament={tournament}
            method={method}
            ask={formation.teams.length > 0}
            disabled={reason !== undefined}
            onDrawn={onDrawn}
          />
        ))}
      </ul>
      {reason && <p className={styles.reason}>{reason}</p>}
    </Section>
  );
}
