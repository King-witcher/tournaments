"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import {
  MenuItem,
  MenuItemDivider,
  MenuItemGroup,
  MenuItemRadioGroup,
  MenuItemSelectable,
} from "@carbon/react";
import { type ReactNode, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { ConfirmModal } from "./confirm-modal";
import type { OverflowAction } from "./overflow-actions";

/** Ação do menu: a mesma do `OverflowActions`, mais ícone, atalho e submenu. */
export type MenuAction = OverflowAction & {
  /** Ícone do `@carbon/icons-react`, à esquerda do texto. */
  icon?: CarbonIconType;
  /** Atalho mostrado à direita; é só a dica, quem usa registra a tecla. */
  shortcut?: string;
  /** Submenu: os itens que abrem ao lado (o item em si não faz nada). */
  items?: MenuEntry[];
};

/** Opção que liga e desliga (`MenuItemSelectable`), com a marca quando ligada. */
export type MenuToggle = {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  icon?: CarbonIconType;
  shortcut?: string;
  disabled?: boolean;
};

/** Uma entre várias opções (`MenuItemRadioGroup`). */
export type MenuChoice = {
  /** Nome do grupo para leitor de tela. */
  label: string;
  options: { value: string; label: string }[];
  value: string;
  onChange: (value: string) => void;
};

/** Itens agrupados (`MenuItemGroup`); o nome é para leitor de tela. */
export type MenuGroup = { group: string; items: MenuEntry[] };

/** Um item do menu; `"divider"` é a linha que separa grupos. */
export type MenuEntry =
  | MenuAction
  | MenuToggle
  | MenuChoice
  | MenuGroup
  | "divider";

type Option = MenuChoice["options"][number];

/**
 * Os itens do `Menu` do Carbon a partir de `MenuEntry` e o `ConfirmModal` das
 * ações com `confirm`, para `MenuButton`, `ComboButton` e `ContextMenu`. Com
 * `href`, o item navega pelo roteador do i18n. Opção e escolha são sempre
 * controladas: o menu desmonta ao fechar e perderia o estado. Ao fechar o
 * Modal, o foco volta para onde o menu o devolveu (o botão que o abriu).
 */
export function useMenuActions(pending?: boolean) {
  const router = useRouter();
  const [asking, setAsking] = useState<MenuAction>();
  const returnTo = useRef<HTMLElement | null>(null);

  const select = (item: MenuAction) => {
    if (item.href !== undefined) router.push(item.href);
    else if (item.confirm) {
      // O Carbon fecha o menu e devolve o foco antes de chamar o item.
      const active = document.activeElement;
      returnTo.current = active instanceof HTMLElement ? active : null;
      setAsking(item);
    } else item.onSelect?.();
  };

  const closeModal = () => {
    setAsking(undefined);
    const target = returnTo.current;
    requestAnimationFrame(() => target?.focus());
  };

  const render = (entries: MenuEntry[]): ReactNode =>
    entries.map((entry, index) => {
      if (entry === "divider") {
        // biome-ignore lint/suspicious/noArrayIndexKey: o divisor não tem nome; a posição o identifica.
        return <MenuItemDivider key={`divider-${index}`} />;
      }
      if ("group" in entry) {
        return (
          <MenuItemGroup key={entry.group} label={entry.group}>
            {render(entry.items)}
          </MenuItemGroup>
        );
      }
      if ("options" in entry) {
        return (
          <MenuItemRadioGroup
            key={entry.label}
            label={entry.label}
            items={entry.options}
            itemToString={(option) => (option as Option).label}
            selectedItem={entry.options.find((o) => o.value === entry.value)}
            onChange={(option) => entry.onChange((option as Option).value)}
          />
        );
      }
      if ("checked" in entry) {
        return (
          <MenuItemSelectable
            key={entry.label}
            label={entry.label}
            selected={entry.checked}
            onChange={entry.onChange}
            renderIcon={entry.icon}
            shortcut={entry.shortcut}
            disabled={entry.disabled || pending}
          />
        );
      }
      return (
        <MenuItem
          key={entry.label}
          label={entry.label}
          kind={entry.destructive ? "danger" : "default"}
          renderIcon={entry.icon}
          shortcut={entry.shortcut}
          disabled={entry.disabled || pending}
          onClick={entry.items ? undefined : () => select(entry)}
        >
          {entry.items && render(entry.items)}
        </MenuItem>
      );
    });

  const confirmModal = (
    <ConfirmModal
      open={asking !== undefined}
      onClose={closeModal}
      onConfirm={() => {
        const item = asking;
        closeModal();
        item?.onSelect?.();
      }}
      title={asking?.confirm?.title ?? asking?.label}
      confirmLabel={asking?.label}
      destructive={asking?.destructive}
      slug={asking?.confirm?.slug}
    >
      {asking?.confirm?.question}
    </ConfirmModal>
  );

  return { renderItems: render, confirmModal };
}
