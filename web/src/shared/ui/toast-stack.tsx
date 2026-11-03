"use client";

import { ToastNotification } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type FocusEvent, useEffect, useState } from "react";

import type { Href } from "@/i18n/navigation";
import { Link } from "./link";
import styles from "./toast-stack.module.scss";

export type ToastItem = {
  id: string;
  /** O que aconteceu, curto. */
  title: string;
  /** Uma frase com o detalhe. */
  subtitle?: string;
  /** Para onde o aviso leva: o aviso inteiro vira este link. Sem ele (a pessoa já está lá), não é clicável. */
  link?: { href: Href; label: string };
};

type Props = {
  items: ToastItem[];
  /** Tira o item da lista (o X ou o tempo). Estável (`useCallback`). */
  onClose: (id: string) => void;
  /** A pessoa fechou o item no X (o tempo não chama), antes do `onClose`: ela viu o aviso. */
  onDismiss?: (id: string) => void;
  /** Nome da região para leitor de tela ("Avisos"). */
  label: string;
  /** Cada item some depois disto (ms), contado de novo quando o ponteiro ou o foco saem da pilha. */
  timeout?: number;
};

/**
 * Avisos que chegam sozinhos (de outra pessoa, do servidor), empilhados no
 * canto de cima, à direita, com o X e, se houver destino, clicáveis por
 * inteiro. Somem depois de `timeout`, mas não enquanto o ponteiro ou o foco
 * estão num deles. A região (`role="log"`) existe sempre, para o leitor de
 * tela anunciar o que entra: monte a pilha uma vez e mude só os `items`. Para
 * "copiado" e afins, use `Toast`.
 */
export function ToastStack({
  items,
  onClose,
  onDismiss,
  label,
  timeout = 10000,
}: Props) {
  // Qual aviso tem o ponteiro e qual tem o foco, não "sim ou não": quando o
  // elemento sai da tela (o X, o link, o aviso sob o ponteiro), `onBlur` e
  // `onPointerLeave` não chegam, e a pausa precisa acabar junto com o aviso.
  const [hovered, setHovered] = useState<string>();
  const [focused, setFocused] = useState<string>();
  const paused = items.some(({ id }) => id === hovered || id === focused);
  return (
    <div
      className={styles.stack}
      // Região viva (educada) que existe antes do aviso: o que entra é lido.
      role="log"
      aria-label={label}
    >
      {items.map((item) => (
        <StackedToast
          key={item.id}
          item={item}
          paused={paused}
          timeout={timeout}
          onClose={onClose}
          onDismiss={onDismiss}
          onHover={setHovered}
          onFocusChange={setFocused}
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
  onDismiss?: (id: string) => void;
  onHover: (id?: string) => void;
  onFocusChange: (id?: string) => void;
};

function StackedToast({
  item,
  paused,
  timeout,
  onClose,
  onDismiss,
  onHover,
  onFocusChange,
}: ToastProps) {
  const t = useTranslations("Common");
  useEffect(() => {
    if (paused) return;
    const timer = setTimeout(() => onClose(item.id), timeout);
    return () => clearTimeout(timer);
  }, [paused, timeout, item.id, onClose]);
  const leaveFocus = (event: FocusEvent<HTMLDivElement>) => {
    if (!event.currentTarget.contains(event.relatedTarget)) onFocusChange();
  };
  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: só observa o ponteiro e o foco, para segurar o tempo; quem interage é o X e o link.
    <div
      className={styles.item}
      onPointerEnter={() => onHover(item.id)}
      onPointerLeave={() => onHover()}
      onFocus={() => onFocusChange(item.id)}
      onBlur={leaveFocus}
    >
      <ToastNotification
        kind="info"
        className={styles.notification}
        role="status"
        lowContrast
        title={item.title}
        subtitle={item.subtitle}
        onClose={() => {
          onDismiss?.(item.id);
          onClose(item.id);
          return false;
        }}
        aria-label={t("close")}
        statusIconDescription={t("notice.info")}
      />
      {/* Fora do ToastNotification, que recusa conteúdo interativo (lança no efeito). */}
      {item.link && (
        <div className={styles.action}>
          <Link href={item.link.href} className={styles.link}>
            {item.link.label}
          </Link>
        </div>
      )}
    </div>
  );
}
