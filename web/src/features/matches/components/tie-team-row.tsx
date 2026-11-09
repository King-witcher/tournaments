import { useTranslations } from "next-intl";

import { Avatar } from "@/shared/ui/avatar";
import type { TieTeam } from "../lib/ties";
import styles from "./tie-team-row.module.scss";

/**
 * Um empatado na lista do Modal: a posição que ele ocupa na ordem de agora, o
 * nome e os números da classificação, com o rótulo em cima de cada um (a lista
 * não tem cabeçalho). No celular, os números descem para baixo do nome.
 */
export function TieTeamRow({
  position,
  team,
}: {
  position: number;
  team: TieTeam;
}) {
  const t = useTranslations("Bracket.table");
  return (
    <div className={styles.row}>
      <span className={styles.position}>{t("place", { position })}</span>
      <span className={styles.team}>
        <Avatar src={team.avatarUrl} name={team.name} />
        <span className={styles.name}>{team.name}</span>
      </span>
      <dl className={styles.numbers}>
        {team.numbers.map((number) => (
          <div key={number.key}>
            <dt className={styles.label}>{number.label}</dt>
            <dd>{number.value}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
