"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import { Button as CarbonButton, Link as CarbonLink } from "@carbon/react";
import { useTranslations } from "next-intl";
import type {
  ComponentProps,
  MouseEvent,
  MouseEventHandler,
  ReactNode,
  Ref,
} from "react";

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

/** @deprecated Use `kind`. */
export type ButtonVariant = "default" | "outline" | "destructive" | "link";

const KIND_OF_VARIANT: Record<Exclude<ButtonVariant, "link">, ButtonKind> = {
  default: "primary",
  outline: "tertiary",
  destructive: "danger",
};

export type ButtonProps = Omit<ComponentProps<"button">, "children"> & {
  children: ReactNode;
  /** Hierarquia: `primary` (uma por área), `secondary`, `tertiary`, `ghost`; `danger*` para o que apaga. */
  kind?: ButtonKind;
  /** @deprecated Use `kind` (`default`→`primary`, `outline`→`tertiary`, `destructive`→`danger`); `link` vira um botão com cara de link. */
  variant?: ButtonVariant;
  /** `md` por padrão. `default` é o nome antigo de `md`. */
  size?: ButtonSize | "default";
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
  kind,
  variant,
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
  const carbonSize = size === "default" ? "md" : size;
  const guard = (event: MouseEvent<HTMLButtonElement>) => {
    if (pending) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
  };
  const busy = {
    "aria-disabled": pending || undefined,
    "aria-busy": pending || undefined,
  };
  const spinner = pending && (
    <Spinner className={styles.spinner} label={t("loading")} />
  );

  if (variant === "link" && !kind) {
    return (
      <CarbonLink
        as="button"
        ref={ref}
        type={type}
        disabled={disabled}
        // O Link do Carbon tipa o clique como de <a>; aqui ele é um <button>.
        onClick={guard as unknown as MouseEventHandler<HTMLAnchorElement>}
        className={cn(pending && styles.pending, className)}
        {...busy}
        {...(rest as object)}
      >
        {children}
        {spinner}
      </CarbonLink>
    );
  }

  return (
    <CarbonButton
      ref={ref}
      kind={
        kind ??
        (variant && variant !== "link" ? KIND_OF_VARIANT[variant] : "primary")
      }
      size={carbonSize}
      type={type}
      disabled={disabled}
      onClick={guard}
      renderIcon={pending ? undefined : renderIcon}
      className={cn(pending && styles.pending, className)}
      {...busy}
      {...rest}
    >
      {children}
      {spinner}
    </CarbonButton>
  );
}
