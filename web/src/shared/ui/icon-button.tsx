"use client";

import { useTranslations } from "next-intl";
import type { ComponentProps, MouseEvent, ReactNode, Ref } from "react";

import { cn } from "@/shared/lib/cn";
import type { ButtonSize } from "./button";
import styles from "./button.module.scss";
import { IconButton as CarbonIconButton } from "./carbon/button";
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
  /**
   * O tooltip deixa o ponteiro passar. Só para botões empilhados em linhas
   * (subir e descer numa lista que se reordena): o balão de uma linha cobre
   * os botões da linha de cima, e o clique neles cairia no balão.
   */
  tooltipPassThrough?: boolean;
  ref?: Ref<HTMLButtonElement>;
};

/**
 * Botão só com ícone (`IconButton` do Carbon): o `label` é o tooltip e o nome
 * acessível. `pending` age como o do `Button`: sem `disabled` (o foco fica),
 * com `aria-disabled`/`aria-busy`, ignora o clique e mostra o indicador no
 * lugar do ícone. Ação com texto é o `Button`. O tooltip muda de lado quando
 * sairia da tela (`autoAlign`: no celular, as ações do `PageHeader` ficam
 * encostadas na margem). Por padrão o balão segura o ponteiro (dá para passar
 * o mouse sobre ele sem que suma); `tooltipPassThrough` troca isso pelo clique
 * no que está embaixo, em listas de linhas com botões.
 *
 * @example
 * <IconButton label={t("up", { name })} tooltipPassThrough onClick={onUp}>
 *   <ArrowUp />
 * </IconButton>
 */
export function IconButton({
  children,
  label,
  kind = "ghost",
  size = "md",
  pending,
  tooltipPassThrough,
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
      autoAlign
      wrapperClasses={tooltipPassThrough ? styles.passThrough : undefined}
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
        <Spinner
          className={cn(
            styles.spinner,
            styles.alone,
            kind === "ghost" && styles.ghostIcon,
          )}
          label={t("loading")}
        />
      ) : (
        children
      )}
    </CarbonIconButton>
  );
}
