import { useTranslations } from "next-intl";
import type { HTMLAttributes, ReactNode } from "react";

import { MemberLink } from "@/features/members/components/member-link";
import { shownNickname } from "@/features/members/lib/nickname";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { IconIndicator } from "@/shared/ui/icon-indicator";
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
  /** Na edição, a alça que arrasta (`DragHandle`). */
  handle?: ReactNode;
  /** O jogador está sendo arrastado: fica só o lugar dele. */
  placeholder?: boolean;
  /** O menu ⋯ do jogador. */
  actions?: ReactNode;
  ref?: (node: HTMLLIElement | null) => void;
};

/**
 * Um jogador na formação dos times, numa linha só: alça, avatar, apelido
 * (o longo termina em reticências), as marcas e o menu. O caso esperado não
 * tem marca: "capitão" é um papel, em texto discreto; o triângulo só em quem
 * não fez o check-in, com a explicação na dica; a etiqueta, só em quem saiu.
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
  placeholder,
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
      <span className={styles.nick} title={name}>
        {name}
      </span>
    </>
  );

  return (
    <li
      className={cn(styles.line, placeholder && styles.placeholder, className)}
      {...rest}
    >
      {handle}
      {link && !left ? (
        <MemberLink tenant={tenant} userId={userId} className={styles.name}>
          {content}
        </MemberLink>
      ) : (
        <span className={styles.name}>{content}</span>
      )}
      {captain && <span className={styles.role}>{t("captain")}</span>}
      {left ? (
        <span className={styles.mark}>
          <StatusTag status="cancelled" size="sm">
            {t("left")}
          </StatusTag>
        </span>
      ) : (
        checkedIn === false && (
          <span className={styles.mark}>
            <IconIndicator
              kind="caution-minor"
              compact
              autoAlign
              align="top"
              label={t("notCheckedInTip")}
              iconDescription={t("notCheckedIn")}
            />
          </span>
        )
      )}
      {actions && <span className={styles.actions}>{actions}</span>}
    </li>
  );
}
