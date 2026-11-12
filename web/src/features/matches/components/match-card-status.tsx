import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { StatusTag } from "@/shared/ui/status-tag";
import { isActionable, taskKey, taskTag } from "../lib/status";
import styles from "./match-card-status.module.scss";

type Props = { match: components["schemas"]["BracketMatch"] };

/**
 * Rodapé do cartão, para achar de relance o que pede atenção: a contestação
 * (a de quem vê, com o que falta, ou um mapa contestado), a ação que falta a
 * quem vê e "Sua série". Esperar (o adversário, o veto, a confirmação) é o
 * caminho normal e não tem etiqueta: o ícone do topo já diz que a série
 * corre. O empate da série decidida é texto. Sem o que dizer, o espaço fica
 * vazio, para os cartões da rodada terem a mesma altura.
 */
export function MatchCardStatus({ match: m }: Props) {
  const t = useTranslations("Bracket");
  const tag = m.task ? taskTag(m.task) : null;

  let content = null;
  if (m.task && tag === "contested") {
    content = (
      <StatusTag status="contested" size="sm">
        {t(`task.${taskKey(m.task, m.veto)}`)}
      </StatusTag>
    );
  } else if (m.contested_maps > 0) {
    content = (
      <StatusTag status="contested" size="sm">
        {t("result.contested")}
      </StatusTag>
    );
  } else if (m.task && isActionable(m.task) && !m.opponent_playing) {
    content = (
      <StatusTag status="action" size="sm">
        {t(`task.${taskKey(m.task, m.veto)}`)}
      </StatusTag>
    );
  } else if (m.yours && m.state === "ready") {
    content = (
      <StatusTag status="neutral" size="sm">
        {t("yours")}
      </StatusTag>
    );
  } else if (m.state === "completed" && m.result && m.result.winner == null) {
    content = <span className={styles.text}>{t("result.drawn")}</span>;
  }
  return <div className={styles.status}>{content}</div>;
}
