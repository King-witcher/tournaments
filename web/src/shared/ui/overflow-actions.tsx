"use client";

import { OverflowMenu, OverflowMenuItem } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ReactNode, useRef, useState } from "react";

import { type Href, useRouter } from "@/i18n/navigation";
import { ConfirmModal } from "./confirm-modal";
import styles from "./overflow-actions.module.scss";
import { Spinner } from "./spinner";

export type OverflowAction = {
  /** Texto do item (e do primário da confirmação). */
  label: string;
  /** O que o item faz; ignorado quando há `href`. */
  onSelect?: () => void;
  /** Leva a outra página. */
  href?: Href;
  /** Apaga ou tira algo: item vermelho e, com `confirm`, Modal `danger`. */
  destructive?: boolean;
  disabled?: boolean;
  /** Pede confirmação antes de `onSelect` (o mesmo Modal do `ConfirmButton`). */
  confirm?: {
    question: ReactNode;
    title?: ReactNode;
    /** Texto que precisa ser digitado para liberar. */
    slug?: string;
  };
};

type Props = {
  items: OverflowAction[];
  /** Nome do menu para leitor de tela; padrão "Mais ações". */
  label?: string;
  /** Uma das ações está em andamento: o indicador aparece ao lado do ⋯. */
  pending?: boolean;
  size?: "sm" | "md" | "lg";
};

/**
 * Menu ⋯ (`OverflowMenu` do Carbon) para o que é raro ou destrutivo, numa
 * linha de tabela ou no cabeçalho da página. Itens com `confirm` abrem o
 * `ConfirmModal` e, ao fechar, o foco volta ao ⋯.
 */
export function OverflowActions({ items, label, pending, size = "md" }: Props) {
  const t = useTranslations("Common");
  const router = useRouter();
  const wrapper = useRef<HTMLSpanElement>(null);
  const [asking, setAsking] = useState<OverflowAction>();

  const focusTrigger = () =>
    requestAnimationFrame(() =>
      wrapper.current?.querySelector("button")?.focus(),
    );

  const select = (item: OverflowAction) => {
    if (item.href !== undefined) router.push(item.href);
    else if (item.confirm) setAsking(item);
    else item.onSelect?.();
  };

  return (
    <span
      ref={wrapper}
      className={styles.wrapper}
      aria-busy={pending || undefined}
    >
      {pending && <Spinner label={t("loading")} />}
      <OverflowMenu
        aria-label={label ?? t("moreActions")}
        iconDescription={label ?? t("moreActions")}
        size={size}
        flipped
      >
        {items.map((item) => (
          <OverflowMenuItem
            key={item.label}
            itemText={item.label}
            isDelete={item.destructive}
            disabled={item.disabled || pending}
            onClick={() => select(item)}
          />
        ))}
      </OverflowMenu>
      <ConfirmModal
        open={asking !== undefined}
        onClose={() => {
          setAsking(undefined);
          focusTrigger();
        }}
        onConfirm={() => {
          const item = asking;
          setAsking(undefined);
          focusTrigger();
          item?.onSelect?.();
        }}
        title={asking?.confirm?.title ?? asking?.label}
        confirmLabel={asking?.label}
        destructive={asking?.destructive}
        slug={asking?.confirm?.slug}
      >
        {asking?.confirm?.question}
      </ConfirmModal>
    </span>
  );
}
