import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import { CodeSnippet as CarbonCodeSnippet } from "./carbon/code-snippet";

type Props = ComponentProps<typeof CarbonCodeSnippet>;

/**
 * Texto para copiar como está (`CodeSnippet` do Carbon), em fonte mono e com
 * o botão de copiar: um link de convite, um id, um comando. `type`: `inline`
 * no meio de uma frase (o trecho inteiro é o botão), `single` numa linha que
 * rola na horizontal (o padrão), `multi` em várias, com "Mostrar mais" acima
 * de 15 linhas (`maxCollapsedNumberOfRows`). `hideCopyButton` tira o botão;
 * `wrapText` quebra as linhas; `copyText` copia outro texto no lugar do que
 * aparece; `disabled` apaga o trecho. Só o botão, ao lado de um texto que
 * já está na tela: `CopyButton`.
 *
 * Traduz a dica "Copiado", o nome do botão de copiar, "Mostrar mais" e
 * "Mostrar menos" e o `aria-label`: no `single` e no `multi`, o nome do
 * trecho ("Link de convite"; o padrão é "Trecho de código"); no `inline`, o
 * nome do botão ("Copiar"). Serve em Server e em Client Component.
 */
export function CodeSnippet({
  type = "single",
  "aria-label": label,
  feedback,
  copyButtonDescription,
  showMoreText,
  showLessText,
  ...props
}: Props) {
  const t = useTranslations("Common");
  return (
    <CarbonCodeSnippet
      type={type}
      aria-label={
        label ?? (type === "inline" ? t("copy.copy") : t("codeSnippet.label"))
      }
      feedback={feedback ?? t("copy.copied")}
      copyButtonDescription={copyButtonDescription ?? t("copy.copy")}
      showMoreText={showMoreText ?? t("codeSnippet.showMore")}
      showLessText={showLessText ?? t("codeSnippet.showLess")}
      {...props}
    />
  );
}
