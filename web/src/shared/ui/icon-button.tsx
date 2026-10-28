"use client";

import { IconButton as CarbonIconButton } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ComponentProps, MouseEvent, ReactNode, Ref } from "react";

import { cn } from "@/shared/lib/cn";
import type { ButtonSize } from "./button";
import styles from "./button.module.scss";
import { Spinner } from "./spinner";

type Props = Omit<ComponentProps<"button">, "children" | "ref" | "title"> & {
  /** O ícone do `@carbon/icons-react` (`<ArrowUp />`). */
  children: ReactNode;
  /** O que o botão faz: vira o tooltip e o nome acessível. Obrigatório, porque sem texto o botão não tem nome. */
  label: string;
  /** `ghost` por padrão: ação de apoio ao lado de um texto ou numa linha. */
  kind?: "primary" | "secondary" | "tertiary" | "ghost";
  /** `md` por padrão; `sm` em linha de tabela ou de lista. */
  size?: ButtonSize;
  /** A ação está em andamento: ignora clique e Enter e troca o ícone pelo indicador, sem `disabled`. */
  pending?: boolean;
  ref?: Ref<HTMLButtonElement>;
};

/**
 * Botão só com ícone (`IconButton` do Carbon): o `label` é o tooltip e o nome
 * acessível. `pending` age como o do `Button`: sem `disabled` (o foco fica),
 * com `aria-disabled`/`aria-busy`, ignora o clique e mostra o indicador no
 * lugar do ícone. Ação com texto é o `Button`.
 */
export function IconButton({
  children,
  label,
  kind = "ghost",
  size = "md",
  pending,
  onClick,
  className,
  type = "button",
  ref,
  ...rest
}: Props) {
  const t = useTranslations("Common");
  const guard = (event: MouseEvent<HTMLButtonElement>) => {
    if (pending) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
  };

  return (
    <CarbonIconButton
      ref={ref}
      label={label}
      kind={kind}
      size={size}
      type={type}
      onClick={guard}
      className={cn(pending && styles.pending, className)}
      aria-disabled={pending || undefined}
      aria-busy={pending || undefined}
      {...rest}
    >
      {pending ? (
        <Spinner className={styles.spinner} label={t("loading")} />
      ) : (
        children
      )}
    </CarbonIconButton>
  );
}
