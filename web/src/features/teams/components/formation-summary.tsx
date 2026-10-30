import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { type Formation, formationCounts, MIN_TEAMS } from "../lib/formation";
import styles from "./formation-summary.module.scss";

type Props = {
  formation: Formation;
  /** O check-in conta para o time estar completo (`checkInShown`). */
  checkIn: boolean;
  /** Já começou: quantos times estão na chave, sem completo ou incompleto. */
  started: boolean;
  /** Os links da página ("Ordem da chave", "Voltar à organização"). */
  links?: ReactNode;
};

/**
 * O resumo da formação no topo: quantos times completos, incompletos e
 * reservas, a regra de iniciar enquanto ela não se cumpre e os links da
 * organização. Depois de iniciar, quantos times estão na chave.
 */
export function FormationSummary({
  formation,
  checkIn,
  started,
  links,
}: Props) {
  const t = useTranslations("Team.summary");
  const counts = formationCounts(formation, checkIn);
  return (
    <div className={styles.summary}>
      <p className={styles.counts}>
        {started
          ? t("started", {
              teams: formation.teams.length,
              reserves: counts.reserves,
            })
          : t("counts", counts)}
      </p>
      {!started && counts.complete < MIN_TEAMS && (
        <p className={styles.rule}>{t("rule", { min: MIN_TEAMS })}</p>
      )}
      {links && <div className={styles.links}>{links}</div>}
    </div>
  );
}
