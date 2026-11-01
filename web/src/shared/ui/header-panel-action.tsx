"use client";

import { HeaderGlobalAction, HeaderPanel } from "@carbon/react";
import {
  type FocusEvent,
  type KeyboardEvent,
  type MouseEvent,
  type ReactNode,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";

import { usePathname } from "@/i18n/navigation";
import { focusSibling } from "@/shared/lib/focus";
import styles from "./header-panel-action.module.scss";

type Props = {
  /** Nome da ação e do painel (leitor de tela e dica ao passar o mouse). */
  label: string;
  /** Ícone de 20px do `@carbon/icons-react` (ou o avatar). */
  icon: ReactNode;
  /**
   * Texto visível ao lado do ícone (o nome de quem está logado), cortado com
   * reticências; o nome acessível continua o `label`.
   */
  text?: string;
  /**
   * Número no canto do ícone (o `BadgeIndicator` do Carbon, até "999+"), que
   * o leitor de tela lê depois do nome; 0 ou nada, sem número.
   */
  badgeCount?: number;
  /**
   * O painel leva a outras páginas (as comunidades): fica num `<nav>`. Os
   * outros ficam num `<section>`, os dois com o `label` como nome.
   */
  navigation?: boolean;
  /** O conteúdo do painel; só existe enquanto ele está aberto. */
  children: ReactNode;
};

/**
 * Ação da barra do cabeçalho que abre um `HeaderPanel` à direita (as
 * comunidades, os avisos, a pessoa). Abre e fecha no clique. Fecha com Esc
 * (dentro do painel, o foco volta ao botão), clicando fora, quando o foco sai
 * do botão e do painel (Tab, outra ação da barra: um painel aberto por vez),
 * num link dele ou ao trocar de página. Aberto pelo teclado, o foco vai para
 * o primeiro item (num painel que carrega ao abrir, quando o item chega, se
 * o foco ainda está no botão); pelo mouse, fica no botão, sem o contorno de
 * foco num item, que pareceria escolhido. ↑ e ↓ andam pelo painel inteiro.
 */
export function HeaderPanelAction({
  label,
  icon,
  text,
  badgeCount,
  navigation,
  children,
}: Props) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  // O clique do Enter e do Espaço tem `detail` 0.
  const byKeyboard = useRef(false);
  const panelId = useId();

  // Trocou de página (um item do painel era link): fecha.
  const pathname = usePathname();
  const [openedAt, setOpenedAt] = useState(pathname);
  if (open && openedAt !== pathname) {
    setOpen(false);
    setOpenedAt(pathname);
  }

  useEffect(() => {
    if (!open) return;
    const focusFirst = () => {
      const first =
        panel.current?.querySelector<HTMLElement>("a, button, input");
      first?.focus();
      return Boolean(first);
    };
    const waiting = new MutationObserver(() => {
      const active = document.activeElement;
      if (active !== trigger.current && active !== document.body) {
        waiting.disconnect();
      } else if (focusFirst()) waiting.disconnect();
    });
    if (byKeyboard.current && !focusFirst() && panel.current) {
      waiting.observe(panel.current, { childList: true, subtree: true });
    }
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (
        !panel.current?.contains(target) &&
        !trigger.current?.contains(target)
      )
        setOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => {
      waiting.disconnect();
      document.removeEventListener("pointerdown", onPointerDown);
    };
  }, [open]);

  // Sem destino (a janela perdeu o foco), fica aberto.
  const closeIfFocusLeaves = (event: FocusEvent) => {
    const next = event.relatedTarget as Node | null;
    if (
      next &&
      !panel.current?.contains(next) &&
      !trigger.current?.contains(next)
    )
      setOpen(false);
  };

  // O HeaderGlobalAction e o HeaderPanel repassam estes atributos e eventos
  // ao elemento, mas os tipos deles não os declaram (o `onClick` do
  // HeaderGlobalAction é tipado sem o evento).
  const triggerProps = {
    "aria-expanded": open,
    "aria-controls": panelId,
    // Do IconButton por baixo: a classe do invólucro da dica e o número.
    wrapperClasses: text ? styles.wrapper : undefined,
    badgeCount: badgeCount || undefined,
    onClick: (event: MouseEvent<HTMLButtonElement>) => {
      byKeyboard.current = event.detail === 0;
      setOpenedAt(pathname);
      setOpen((value) => !value);
    },
    onBlur: closeIfFocusLeaves,
    onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => {
      if (event.key === "Escape") setOpen(false);
    },
  } as object;
  const panelProps = {
    id: panelId,
    onBlur: closeIfFocusLeaves,
    onKeyDown: (event: KeyboardEvent<HTMLDivElement>) => {
      if (event.key !== "Escape") return;
      setOpen(false);
      trigger.current?.focus();
    },
    // Na captura, antes do Switcher do Carbon, cujas setas só andam dentro
    // da própria lista.
    onKeyDownCapture: (event: KeyboardEvent<HTMLDivElement>) => {
      const step =
        event.key === "ArrowDown" ? 1 : event.key === "ArrowUp" ? -1 : 0;
      if (!step || (event.target as Element).closest("input, textarea")) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      focusSibling(event.currentTarget, step);
    },
    // Também o link da página atual (a comunidade em que se está), que não
    // troca de página.
    onClick: (event: MouseEvent<HTMLDivElement>) => {
      if ((event.target as Element).closest("a")) setOpen(false);
    },
  } as object;
  const Region = navigation ? "nav" : "section";

  return (
    <>
      <HeaderGlobalAction
        ref={trigger}
        aria-label={label}
        className={text ? styles.withText : undefined}
        isActive={open}
        tooltipAlignment="end"
        {...triggerProps}
      >
        {icon}
        {text && <span className={styles.text}>{text}</span>}
      </HeaderGlobalAction>
      <HeaderPanel
        ref={panel}
        expanded={open}
        addFocusListeners={false}
        {...panelProps}
      >
        {open && <Region aria-label={label}>{children}</Region>}
      </HeaderPanel>
    </>
  );
}
