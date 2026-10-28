"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import { Button as CarbonButton } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ComponentProps, MouseEvent, ReactNode, Ref } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./button.module.scss";
import { Spinner } from "./spinner";

export type ButtonKind =
  | "primary"
  | "secondary"
  | "tertiary"
  | "ghost"
  | "danger"
  | "danger--tertiary"
  | "danger--ghost";

/** Tamanho do Carbon; abaixo de `md` (celular) todos ficam com 48px de altura. */
export type ButtonSize = "sm" | "md" | "lg";

export type ButtonProps = Omit<ComponentProps<"button">, "children"> & {
  children: ReactNode;
  /** Hierarquia: `primary` (uma por área), `secondary`, `tertiary`, `ghost`; `danger*` para o que apaga. */
  kind?: ButtonKind;
  /** `md` por padrão. */
  size?: ButtonSize;
  /** A ação está em andamento: ignora clique e Enter e mostra o indicador girando. */
  pending?: boolean;
  /** Ícone do `@carbon/icons-react`, à direita do texto. */
  renderIcon?: CarbonIconType;
  ref?: Ref<HTMLButtonElement>;
};

/**
 * Botão do Carbon. `pending`: a ação está em andamento; ignora o clique (sem
 * clique duplo, nem pelo Enter num campo do formulário) e mostra o
 * `InlineLoading` dentro do botão, para quem clicou saber que algo vai
 * acontecer. Pendente, ele não recebe `disabled`: o navegador tiraria o foco
 * dele, e quem usa teclado voltaria para o começo da página a cada ação.
 */
export function Button({
  kind = "primary",
  size = "md",
  pending,
  disabled,
  onClick,
  className,
  type = "button",
  renderIcon,
  children,
  ref,
  ...rest
}: ButtonProps) {
  const t = useTranslations("Common");
  const guard = (event: MouseEvent<HTMLButtonElement>) => {
    if (pending) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
  };

  return (
    <CarbonButton
      ref={ref}
      kind={kind}
      size={size}
      type={type}
      disabled={disabled}
      onClick={guard}
      renderIcon={pending ? undefined : renderIcon}
      className={cn(pending && styles.pending, className)}
      aria-disabled={pending || undefined}
      aria-busy={pending || undefined}
      {...rest}
    >
      {children}
      {pending && <Spinner className={styles.spinner} label={t("loading")} />}
    </CarbonButton>
  );
}
