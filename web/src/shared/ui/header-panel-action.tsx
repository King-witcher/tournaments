"use client";

import { HeaderGlobalAction, HeaderPanel } from "@carbon/react";
import {
  type FocusEvent,
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";

import { usePathname } from "@/i18n/navigation";

type Props = {
  /** Nome da ação (leitor de tela e dica ao passar o mouse). */
  label: string;
  /** Ícone de 20px do `@carbon/icons-react` (ou o avatar). */
  icon: ReactNode;
  /** O conteúdo do painel; só existe enquanto ele está aberto. */
  children: ReactNode;
};

/**
 * Ação da barra do cabeçalho que abre um `HeaderPanel` à direita (idioma, a
 * pessoa). Abre e fecha no clique; fecha com Esc (o foco volta ao botão),
 * clicando fora, saindo dele pelo Tab ou ao trocar de página. Ao abrir, o
 * foco vai para o primeiro item.
 */
export function HeaderPanelAction({ label, icon, children }: Props) {
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
    panel.current?.querySelector<HTMLElement>("a, button, input")?.focus();
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (
        !panel.current?.contains(target) &&
        !trigger.current?.contains(target)
      )
        setOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, [open]);

  // O HeaderGlobalAction e o HeaderPanel repassam estes atributos ao
  // elemento, mas os tipos deles não os declaram.
  const triggerProps = {
    "aria-expanded": open,
    "aria-controls": panelId,
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
  } as object;

  return (
    <>
      <HeaderGlobalAction
        ref={trigger}
        aria-label={label}
        isActive={open}
        tooltipAlignment="end"
        onClick={() => {
          setOpenedAt(pathname);
          setOpen((value) => !value);
        }}
        {...triggerProps}
      >
        {icon}
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
