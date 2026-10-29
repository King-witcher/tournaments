import { Draggable } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import type { HTMLAttributes, ReactNode } from "react";

import { MemberLink } from "@/features/members/components/member-link";
import { shownNickname } from "@/features/members/lib/nickname";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { StatusTag } from "@/shared/ui/status-tag";
import styles from "./player-line.module.scss";

type Props = Omit<HTMLAttributes<HTMLLIElement>, "children"> & {
  tenant: string;
  userId: number;
  /** O apelido como a API manda (vazio: conta excluída). */
  nickname: string;
  avatarUrl?: string | null;
  /** O apelido leva à página da pessoa (só leitura, e só para quem continua na comunidade). */
  link?: boolean;
  /** Marca "capitão". */
  captain?: boolean;
  /** Com o check-in valendo, se a pessoa fez; `undefined` não mostra nada. */
  checkedIn?: boolean;
  /** Saiu da comunidade (continua no time como histórico). */
  left?: boolean;
  /** Na edição, a alça que diz que a linha se arrasta (só no computador). */
  handle?: boolean;
  /** O menu ⋯ do jogador. */
  actions?: ReactNode;
};

/**
 * Um jogador na formação dos times: avatar, apelido, as marcas (capitão,
 * check-in, saiu) e o menu dele. É o `<li>` da lista do time ou dos
 * reservas; na edição, recebe as props de arrastar.
 */
export function PlayerLine({
  tenant,
  userId,
  nickname,
  avatarUrl,
  link,
  captain,
  checkedIn,
  left,
  handle,
  actions,
  className,
  ...rest
}: Props) {
  const t = useTranslations("Team.member");
  const tMember = useTranslations("Member");
  const name = shownNickname(nickname, tMember("deletedAccount"));
  const content = (
    <>
      <Avatar src={avatarUrl} name={name} />
      {name}
    </>
  );

  return (
    <li
      className={cn(styles.line, handle && styles.draggable, className)}
      {...rest}
    >
      {handle && <Draggable aria-hidden className={styles.handle} />}
      {link && !left ? (
        <MemberLink tenant={tenant} userId={userId} className={styles.name}>
          {content}
        </MemberLink>
      ) : (
        <span className={styles.name}>{content}</span>
      )}
      <span className={styles.tags}>
        {captain && (
          <StatusTag status="neutral" size="sm">
            {t("captain")}
          </StatusTag>
        )}
        {left ? (
          <StatusTag status="cancelled" size="sm">
            {t("left")}
          </StatusTag>
        ) : (
          checkedIn !== undefined && (
            <StatusTag status={checkedIn ? "confirmed" : "check_in"} size="sm">
              {checkedIn ? t("checkedIn") : t("notCheckedIn")}
            </StatusTag>
          )
        )}
      </span>
      {actions && <span className={styles.actions}>{actions}</span>}
    </li>
  );
}
