"use client";

import {
  ActionableNotification as CarbonActionableNotification,
  FeatureFlags,
} from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonActionableNotification>;

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
 * Aviso que pede uma resposta agora (`ActionableNotification` do Carbon), com
 * o botão de ação (`actionButtonLabel` + `onActionButtonClick`) e o de
 * fechar. Recebe o foco ao aparecer e o prende até a pessoa agir ou fechar
 * (`role="alertdialog"`; Esc fecha): use só quando a ação não pode esperar.
 * `inline` fica no fluxo da página, junto do que trata; sem ele, tem o
 * formato de toast (a posição fixa é de quem usa). Só informar: `Notice` na
 * página ou `Toast` passageiro; destaque que carrega com a página e não se
 * fecha: `Callout`.
 *
 * Traduz o nome do botão de fechar e o do ícone de estado, e prende o foco
 * sem os elementos sentinela do Carbon (que o leitor de tela lê em inglês).
 */
export function ActionableNotification({
  kind = "error",
  "aria-label": closeLabel,
  statusIconDescription,
  ...props
}: Props) {
  const t = useTranslations("Common");
  return (
    <FeatureFlags enableFocusWrapWithoutSentinels>
      <CarbonActionableNotification
        kind={kind}
        aria-label={closeLabel ?? t("notification.close")}
        statusIconDescription={
          statusIconDescription ?? t(`notice.${STATUS[kind]}`)
        }
        {...props}
      />
    </FeatureFlags>
  );
}
