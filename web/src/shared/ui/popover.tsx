"use client";

import type { PopoverProps } from "@carbon/react";
import {
  type ComponentProps,
  cloneElement,
  type KeyboardEvent,
  type MouseEvent,
  type ReactElement,
  type ReactNode,
  useState,
} from "react";

import {
  Popover as CarbonPopover,
  PopoverContent as CarbonPopoverContent,
} from "./carbon/popover";
import styles from "./popover.module.scss";

type TriggerProps = {
  onClick?: (event: MouseEvent<HTMLElement>) => void;
  onKeyDown?: (event: KeyboardEvent<HTMLElement>) => void;
  "aria-expanded"?: boolean;
};

type Props = Omit<PopoverProps<"span">, "open" | "onRequestClose" | "as"> & {
  /**
   * O botão que abre e fecha (`Button`, `IconButton` ou `<button>`): ganha o
   * clique, o Esc e o `aria-expanded`. Com `autoAlign`, precisa aceitar `ref`.
   */
  trigger: ReactElement<TriggerProps>;
  /** O `PopoverContent`. */
  children: ReactNode;
  /** Aberto agora, quando quem usa controla; sem ele, o estado fica aqui. */
  open?: boolean;
  /** Aberto ao montar, sem controle de fora. */
  defaultOpen?: boolean;
  /** Pedido de abrir ou fechar: o botão, o clique fora, o Esc, o foco que sai. */
  onOpenChange?: (open: boolean) => void;
};

/**
 * Caixa que abre presa a um botão (`Popover` do Carbon), com conteúdo que a
 * pessoa usa ali mesmo: uma lista curta, opções de exibição, os detalhes de
 * uma etiqueta. Fecha no clique fora, no Esc (o foco volta ao botão) e
 * quando o foco sai. `align` escolhe o lado (`autoAlign` troca se não
 * couber, útil no celular); `caret`, `dropShadow`, `highContrast` e `border`
 * mudam a caixa; `isTabTip` é o formato de aba, colado ao botão. Explicação
 * curta de um termo: `InfoTip` (Toggletip); nome de um botão: `Tooltip`;
 * lista de ações: `OverflowActions`; formulário: `FormModal`.
 *
 * O Carbon só desenha a caixa: o componente guarda o aberto (ou segue
 * `open`), liga o botão e fecha no Esc também com o foco nele.
 *
 * ```tsx
 * <Popover trigger={<Button kind="ghost">+3 mapas</Button>}>
 *   <PopoverContent>…</PopoverContent>
 * </Popover>
 * ```
 */
export function Popover({
  trigger,
  children,
  open: controlled,
  defaultOpen = false,
  onOpenChange,
  ...props
}: Props) {
  const [own, setOwn] = useState(defaultOpen);
  const open = controlled ?? own;
  const change = (next: boolean) => {
    if (next === open) return;
    setOwn(next);
    onOpenChange?.(next);
  };

  // O Esc do Carbon só vale com o foco dentro da caixa; aqui vale no botão.
  const button = cloneElement(trigger, {
    "aria-expanded": open,
    onClick: (event: MouseEvent<HTMLElement>) => {
      trigger.props.onClick?.(event);
      change(!open);
    },
    onKeyDown: (event: KeyboardEvent<HTMLElement>) => {
      trigger.props.onKeyDown?.(event);
      if (event.key === "Escape" && open) {
        event.stopPropagation();
        change(false);
      }
    },
  });

  return (
    <CarbonPopover {...props} open={open} onRequestClose={() => change(false)}>
      {button}
      {children}
    </CarbonPopover>
  );
}

type ContentProps = ComponentProps<typeof CarbonPopoverContent>;

/**
 * O conteúdo do `Popover`, com o respiro do `Toggletip`. Não repassa `ref`:
 * com `autoAlign`, o Carbon põe um `ref` em cada filho que não é o
 * `PopoverContent` dele, para achar o botão, e a caixa viraria a âncora.
 */
export function PopoverContent({
  ref: _anchor,
  children,
  ...props
}: ContentProps) {
  return (
    <CarbonPopoverContent {...props}>
      <div className={styles.content}>{children}</div>
    </CarbonPopoverContent>
  );
}
