import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import styles from "./member-name.module.scss";

type Props = {
  member: Pick<
    components["schemas"]["Member"],
    "nickname" | "display_name" | "avatar_url"
  >;
};

/**
 * O membro numa linha de tabela: avatar, apelido e, quando é outro, o nome
 * exibido embaixo (o apelido pode repetir na comunidade; o avatar e o nome
 * desfazem a dúvida).
 */
export function MemberName({ member }: Props) {
  return (
    <span className={styles.member}>
      <Avatar src={member.avatar_url} name={member.nickname} />
      <span className={styles.names}>
        <span>{member.nickname}</span>
        {member.display_name !== member.nickname && (
          <span className={styles.secondary}>{member.display_name}</span>
        )}
      </span>
    </span>
  );
}
