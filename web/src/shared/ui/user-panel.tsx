"use client";

import { UserAvatar } from "@carbon/icons-react";
import {
  InlineLoading,
  Switcher,
  SwitcherDivider,
  SwitcherItem,
} from "@carbon/react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import { Avatar } from "./avatar";
import { HeaderPanelAction } from "./header-panel-action";
import { ThemeSwitcher } from "./theme-switcher";
import styles from "./user-panel.module.scss";

export type UserPanelItem =
  | { label: string; href: Href }
  | { label: string; onSelect: () => void; pending?: boolean };

type Props = {
  /** Nome da ação: "Conta de Fulano", ou "Entrar ou criar conta" sem sessão. */
  label: string;
  /**
   * Com sessão: o nome na barra (cortado) e no topo do painel (inteiro,
   * quebrando a linha se for longo).
   */
  name?: string;
  avatarSrc?: string | null;
  items: UserPanelItem[];
};

/**
 * O SwitcherItem repassa `as` ao Link do UI Shell (que o aceita), mas o tipo
 * dele não declara `as` e tipa `href` como string.
 */
function asIntlLink(href: Href) {
  return { as: IntlLink, href } as unknown as { href: string };
}

/**
 * O lugar da pessoa no cabeçalho: avatar (ou o ícone de usuário) e, com
 * sessão, o nome, que abrem um painel com os itens (Conta e Sair; sem
 * sessão, Entrar e Criar conta) e, no fim, o tema de cor, que vale com ou
 * sem sessão. Na barra o nome longo termina em reticências (no celular, mais
 * cedo); no painel ele aparece inteiro. Os itens vêm de `features/auth`.
 */
export function UserPanel({ label, name, avatarSrc, items }: Props) {
  return (
    <HeaderPanelAction
      label={label}
      text={name}
      icon={
        avatarSrc ? (
          <Avatar src={avatarSrc} name={name} size="sm" />
        ) : (
          <UserAvatar size={20} />
        )
      }
    >
      {name && <p className={styles.name}>{name}</p>}
      <Switcher aria-label={label} expanded>
        {items.map((item) =>
          "href" in item ? (
            <SwitcherItem
              key={item.label}
              {...asIntlLink(item.href)}
              aria-label={item.label}
            >
              {item.label}
            </SwitcherItem>
          ) : (
            <SwitcherItem
              key={item.label}
              {...({ as: "button", type: "button" } as object)}
              className={styles.action}
              aria-label={item.label}
              aria-disabled={item.pending || undefined}
              aria-busy={item.pending || undefined}
              onClick={() => {
                if (!item.pending) item.onSelect();
              }}
            >
              {item.pending ? (
                <InlineLoading status="active" description={item.label} />
              ) : (
                item.label
              )}
            </SwitcherItem>
          ),
        )}
      </Switcher>
      <SwitcherDivider />
      <ThemeSwitcher />
    </HeaderPanelAction>
  );
}
