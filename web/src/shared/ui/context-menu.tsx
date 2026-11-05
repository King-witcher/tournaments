"use client";

import type { ComponentProps, RefObject } from "react";

import { Menu, useContextMenu } from "./carbon/menu";
import { type MenuEntry, useMenuActions } from "./use-menu-actions";

type Props = Omit<
  ComponentProps<typeof Menu>,
  "children" | "open" | "x" | "y" | "onClose" | "label"
> & {
  /** O elemento (ou a ref dele) que abre o menu com o clique direito. */
  trigger: Element | RefObject<Element | null>;
  /** Nome do menu para leitor de tela. */
  label: string;
  /** Os itens, como no `MenuButton`: ação, opção, escolha, grupo ou `"divider"`. */
  items: MenuEntry[];
};

/**
 * Menu do clique direito (`Menu` do Carbon aberto pelo `useContextMenu`) sobre
 * um elemento, com ações daquele conteúdo. É atalho: no toque e para quem
 * não usa o clique direito ele não existe, então toda ação dele também fica
 * num lugar visível (botão, `MenuButton` ou `OverflowActions`). Os itens
 * seguem as regras do `MenuButton` (`href`, `confirm`, `destructive`, opção
 * marcada e escolha entre várias).
 */
export function ContextMenu({ trigger, label, items, ...rest }: Props) {
  // O tipo do Carbon recusa a ref, que o código dele aceita (`trigger.current`).
  const menu = useContextMenu(trigger as Parameters<typeof useContextMenu>[0]);
  const { renderItems, confirmModal } = useMenuActions();

  return (
    <>
      <Menu {...rest} {...menu} label={label}>
        {renderItems(items)}
      </Menu>
      {confirmModal}
    </>
  );
}
