import { useTranslations } from "next-intl";

import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import styles from "./opponent-name.module.scss";

type Props = {
  team: components["schemas"]["Team"];
  /** O nome já resolvido (`useTeamName`). */
  name: string;
};

/**
 * O adversário numa linha da página da pessoa: o avatar (no duelo) e o nome
 * do time, com "saiu" quando ninguém dele continua na comunidade (a conta
 * excluída já diz isso no nome).
 */
export function OpponentName({ team, name }: Props) {
  const t = useTranslations("Member");
  const solo = soloMember(team);
  const left = teamLeft(team) && team.members.some((m) => m.nickname !== "");
  return (
    <span className={styles.opponent}>
      <Avatar src={solo?.avatar_url} name={name} />
      {left ? t("left", { name }) : name}
    </span>
  );
}
