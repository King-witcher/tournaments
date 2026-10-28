import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { StatusTag } from "@/shared/ui/status-tag";
import { TASK_STATUS } from "../lib/status";
import styles from "./match-card-status.module.scss";

type Props = { match: components["schemas"]["BracketMatch"] };

/**
 * Rodapé do cartão: o que a partida espera de quem vê ou, para os outros, se
 * há mapa esperando confirmação ou contestado. Sem o que dizer, o espaço
 * fica vazio, para os cartões da rodada terem a mesma altura.
 */
export function MatchCardStatus({ match: m }: Props) {
  const t = useTranslations("Bracket");

  let tag = null;
  if (m.task) {
    tag = (
      <StatusTag status={TASK_STATUS[m.task]} size="sm">
        {t(`task.${m.task}`)}
      </StatusTag>
    );
  } else if (m.contested_maps > 0) {
    tag = (
      <StatusTag status="contested" size="sm">
        {t("result.contested")}
      </StatusTag>
    );
  } else if (m.pending_maps > 0) {
    tag = (
      <StatusTag status="waiting" size="sm">
        {t("result.pending")}
      </StatusTag>
    );
  } else if (m.yours && m.state === "ready") {
    tag = (
      <StatusTag status="neutral" size="sm">
        {t("yours")}
      </StatusTag>
    );
  }
  return <div className={styles.status}>{tag}</div>;
}
