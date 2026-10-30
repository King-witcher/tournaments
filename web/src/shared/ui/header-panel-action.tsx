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
import styles from "./header-panel-action.module.scss";

type Props = {
  /** Nome da ação (leitor de tela e dica ao passar o mouse). */
  label: string;
  /** Ícone de 20px do `@carbon/icons-react` (ou o avatar). */
  icon: ReactNode;
  /**
   * Texto visível ao lado do ícone (o nome de quem está logado), cortado com
   * reticências; o nome acessível continua o `label`.
   */
  text?: string;
  /** O conteúdo do painel; só existe enquanto ele está aberto. */
  children: ReactNode;
};

/**
 * Ação da barra do cabeçalho que abre um `HeaderPanel` à direita (idioma, a
 * pessoa). Abre e fecha no clique; fecha com Esc (o foco volta ao botão),
 * clicando fora, saindo dele pelo Tab, num link dele ou ao trocar de página.
 * Ao abrir, o foco vai para o primeiro item; num painel que carrega ao abrir
 * (as comunidades), quando o primeiro item chega, se o foco ainda está no
 * botão.
 */
export function HeaderPanelAction({ label, icon, text, children }: Props) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
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
    if (!focusFirst() && panel.current) {
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

  // O HeaderGlobalAction e o HeaderPanel repassam estes atributos ao
  // elemento, mas os tipos deles não os declaram.
  const triggerProps = {
    "aria-expanded": open,
    "aria-controls": panelId,
    // A classe do invólucro da dica, que o IconButton por baixo aceita.
    wrapperClasses: text ? styles.wrapper : undefined,
  } as object;
  const panelProps = {
    id: panelId,
    "aria-label": label,
    onBlur: (event: FocusEvent<HTMLDivElement>) => {
      const next = event.relatedTarget as Node | null;
      if (
        next &&
        !event.currentTarget.contains(next) &&
        !trigger.current?.contains(next)
      )
        setOpen(false);
    },
    onKeyDown: (event: KeyboardEvent<HTMLDivElement>) => {
      if (event.key !== "Escape") return;
      setOpen(false);
      trigger.current?.focus();
    },
    // Também o link da página atual (a comunidade em que se está), que não
    // troca de página.
    onClick: (event: MouseEvent<HTMLDivElement>) => {
      if ((event.target as Element).closest("a")) setOpen(false);
    },
  } as object;

  return (
    <>
      <HeaderGlobalAction
        ref={trigger}
        aria-label={label}
        className={text ? styles.withText : undefined}
        isActive={open}
        tooltipAlignment="end"
        onClick={() => {
          setOpenedAt(pathname);
          setOpen((value) => !value);
        }}
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
        {open && children}
      </HeaderPanel>
    </>
  );
}
