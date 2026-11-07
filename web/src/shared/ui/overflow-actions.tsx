"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useRef, useState } from "react";

import { type Href, useRouter } from "@/i18n/navigation";
import { OverflowMenu, OverflowMenuItem } from "./carbon/overflow-menu";
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
  /** Apaga ou tira algo: item vermelho, por último e com uma linha antes; com `confirm`, Modal `danger`. */
  destructive?: boolean;
  disabled?: boolean;
  /** Pede confirmação antes de `onSelect` (o mesmo Modal do `ConfirmButton`). */
  confirm?: {
    question: ReactNode;
    title?: ReactNode;
    /** Texto que precisa ser digitado para liberar. */
    slug?: string;
    /** Campos no Modal (uma nota), controlados por quem monta o menu. */
    extra?: ReactNode;
    /** O Modal fechou sem confirmar (limpar o `extra`). */
    onCancel?: () => void;
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
 * `ConfirmModal` e, ao fechar, o foco volta ao ⋯. O tooltip do ⋯ tem
 * `autoAlign`, como o do `IconButton`: muda de lado quando sairia da tela e,
 * posicionado em relação à janela, não alarga um contêiner com rolagem (a
 * coluna dos reservas no montador de times).
 */
export function OverflowActions({ items, label, pending, size = "md" }: Props) {
  const t = useTranslations("Common");
  const router = useRouter();
  const wrapper = useRef<HTMLSpanElement>(null);
  // O item de agora com o mesmo nome (o `extra` controlado muda a cada
  // tecla); se ele sumiu num refresh, o guardado ao abrir, e o Modal fica.
  const [asked, setAsked] = useState<OverflowAction>();
  const asking =
    asked && (items.find((item) => item.label === asked.label) ?? asked);

  const focusTrigger = () =>
    requestAnimationFrame(() =>
      wrapper.current?.querySelector("button")?.focus(),
    );

  const select = (item: OverflowAction) => {
    if (item.href !== undefined) router.push(item.href);
    else if (item.confirm) setAsked(item);
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
        autoAlign
      >
        {items.map((item, index) => (
          <OverflowMenuItem
            key={item.label}
            itemText={item.label}
            isDelete={item.destructive}
            // O Carbon separa com uma linha o que mexe nos dados da pessoa
            // (excluir, tirar) do resto do menu.
            hasDivider={
              item.destructive && index > 0 && !items[index - 1].destructive
            }
            disabled={item.disabled || pending}
            onClick={() => select(item)}
          />
        ))}
      </OverflowMenu>
      <ConfirmModal
        open={asking !== undefined}
        onClose={() => {
          asking?.confirm?.onCancel?.();
          setAsked(undefined);
          focusTrigger();
        }}
        onConfirm={() => {
          const item = asking;
          setAsked(undefined);
          focusTrigger();
          item?.onSelect?.();
        }}
        title={asking?.confirm?.title ?? asking?.label}
        confirmLabel={asking?.label}
        destructive={asking?.destructive}
        slug={asking?.confirm?.slug}
        extra={asking?.confirm?.extra}
      >
        {asking?.confirm?.question}
      </ConfirmModal>
    </span>
  );
}
