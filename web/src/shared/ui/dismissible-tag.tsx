"use client";

import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import { DismissibleTag as CarbonDismissibleTag } from "./carbon/tag";

type Props = ComponentProps<typeof CarbonDismissibleTag>;

/**
 * Etiqueta com um X para tirar (`DismissibleTag` do Carbon): um filtro
 * aplicado ou um item escolhido que a pessoa pode remover (um mapa da pool,
 * um filtro da busca). `onClose` remove: o Carbon não esconde a etiqueta
 * sozinho. `text` é o texto; `type`, a cor; `size`, `sm` | `md` | `lg`;
 * `renderIcon`, o ícone antes do texto. Etiqueta só de leitura: `StatusTag`;
 * que liga e desliga: `SelectableTag`; que abre algo: `OperationalTag`.
 *
 * Traduz o nome do X e a dica dele quando o texto aparece cortado.
 */
export function DismissibleTag({
  title,
  dismissTooltipLabel,
  ...props
}: Props) {
  const t = useTranslations("Common.tag");
  return (
    <CarbonDismissibleTag
      title={title ?? t("dismiss")}
      dismissTooltipLabel={dismissTooltipLabel ?? t("dismissTag")}
      {...props}
    />
  );
}
