import { useTranslations } from "next-intl";
import { type ComponentProps, useId } from "react";

import { Callout as CarbonCallout } from "./carbon/notification";

type Props = ComponentProps<typeof CarbonCallout>;

/** O nome do ícone de cada tipo: `-alt` e `-square` são o mesmo estado. */
const STATUS = {
  error: "error",
  success: "success",
  warning: "warning",
  "warning-alt": "warning",
  info: "info",
  "info-square": "info",
} as const;

/**
 * Destaque fixo (`Callout` do Carbon, antes `StaticNotification`): informação
 * ou cuidado que carrega com o conteúdo, fica no lugar e não se fecha, como a
 * regra que vale para a seção inteira. Os tipos são `info` e `warning` (os
 * outros o Carbon marca como obsoletos). Pode ter um botão
 * (`actionButtonLabel`) e link no texto, com `aria-describedby` igual ao
 * `titleId`. Erro ou retorno de uma ação: `Notice`; o que pede resposta
 * agora: `ActionableNotification`.
 *
 * Gera o `titleId` (o botão de ação aponta para ele) e traduz o nome do
 * ícone. Serve em Server e em Client Component.
 */
export function Callout({
  kind = "info",
  titleId,
  statusIconDescription,
  ...props
}: Props) {
  const t = useTranslations("Common.notice");
  const generated = useId();
  return (
    <CarbonCallout
      kind={kind}
      titleId={titleId ?? generated}
      statusIconDescription={statusIconDescription ?? t(STATUS[kind])}
      {...props}
    />
  );
}
