"use client";

import { ToastNotification } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type FocusEvent, type ReactNode, useEffect, useState } from "react";

import styles from "./toast-stack.module.scss";

export type ToastItem = {
  id: string;
  /** O que aconteceu, curto. */
  title: string;
  /** Uma frase com o detalhe. */
  subtitle?: string;
  /** Um link para onde resolver (`Link` de `shared/ui`). */
  action?: ReactNode;
};

type Props = {
  items: ToastItem[];
  /** Tira o item da lista (o X ou o tempo). Estável (`useCallback`). */
  onClose: (id: string) => void;
  /** Nome da região para leitor de tela ("Avisos"). */
  label: string;
  /** Cada item some depois disto (ms), contado de novo quando o ponteiro ou o foco saem da pilha. */
  timeout?: number;
};

/**
 * Avisos que chegam sozinhos (de outra pessoa, do servidor), empilhados no
 * canto de baixo, com o X e, se houver, o link para resolver. Somem depois de
 * `timeout`, mas não enquanto o ponteiro ou o foco estão na pilha (quem vai
 * clicar no link não perde o aviso). A região (`role="log"`) existe sempre,
 * para o leitor de tela anunciar o que entra: monte a pilha uma vez e mude só
 * os `items`. Para "copiado" e afins, use `Toast`.
 */
export function ToastStack({ items, onClose, label, timeout = 20000 }: Props) {
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const leaveFocus = (event: FocusEvent<HTMLDivElement>) => {
    if (!event.currentTarget.contains(event.relatedTarget)) setFocused(false);
  };
  return (
    <div
      className={styles.stack}
      // Região viva (educada) que existe antes do aviso: o que entra é lido.
      role="log"
      aria-label={label}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
      onFocus={() => setFocused(true)}
      onBlur={leaveFocus}
    >
      {items.map((item) => (
        <StackedToast
          key={item.id}
          item={item}
          paused={hovered || focused}
          timeout={timeout}
          onClose={onClose}
        />
      ))}
    </div>
  );
}

type ToastProps = {
  item: ToastItem;
  paused: boolean;
  timeout: number;
  onClose: (id: string) => void;
};

function StackedToast({ item, paused, timeout, onClose }: ToastProps) {
  const t = useTranslations("Common");
  useEffect(() => {
    if (paused) return;
    const timer = setTimeout(() => onClose(item.id), timeout);
    return () => clearTimeout(timer);
  }, [paused, timeout, item.id, onClose]);
  return (
    // O link fica fora do ToastNotification, colado embaixo: o Carbon recusa
    // conteúdo interativo dentro dele (lança no efeito, também no build de
    // produção, e a página cai).
    <div className={styles.item}>
      <ToastNotification
        kind="info"
        role="status"
        lowContrast
        title={item.title}
        subtitle={item.subtitle}
        onClose={() => {
          onClose(item.id);
          return false;
        }}
        aria-label={t("close")}
        statusIconDescription={t("notice.info")}
      />
      {item.action && <div className={styles.action}>{item.action}</div>}
    </div>
  );
}
