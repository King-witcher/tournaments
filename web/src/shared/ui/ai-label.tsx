import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import {
  AILabel as CarbonAILabel,
  AILabelContent as CarbonAILabelContent,
} from "./carbon/ai-label";

type Props = ComponentProps<typeof CarbonAILabel>;

/**
 * Marca "IA" (`AILabel` do Carbon) ao lado do que foi gerado ou sugerido por
 * IA, que abre a explicação ao clicar (um `AILabelContent` dentro): o que a
 * IA fez e com base em quê. `kind="inline"` com `textLabel` põe um texto ao
 * lado da marca; `size`, de `mini` a `xl`, segue o controle ao lado;
 * `revertActive` troca a marca pelo botão de voltar ao conteúdo da IA
 * (`onRevertClick`), depois que a pessoa editou. O site ainda não usa IA.
 *
 * Traduz "AI", o nome do botão e o de voltar. O Carbon dá preferência ao
 * `slugLabel` (obsoleto, com padrão em inglês) sobre o `aria-label`, por isso
 * os dois recebem o mesmo texto. Como `decorator` de outro componente do
 * Carbon, passe o `size` (`mini` em campo, `xs` em tile): ele só ajusta
 * sozinho o `AILabel` dele.
 */
export function AILabel({
  aiText,
  "aria-label": label,
  slugLabel,
  revertLabel,
  ...props
}: Props) {
  const t = useTranslations("Common.aiLabel");
  const name = label ?? slugLabel ?? t("showInformation");
  return (
    <CarbonAILabel
      aiText={aiText ?? t("text")}
      aria-label={name}
      slugLabel={name}
      revertLabel={revertLabel ?? t("revert")}
      {...props}
    />
  );
}

/**
 * A explicação que o `AILabel` abre: título, texto e, no fim, as ações
 * (`AILabelActions` do Carbon). É o do Carbon sem mudança: ele não tem texto
 * próprio, e o `Toggletip` do `AILabel` depende de ser este componente.
 */
export const AILabelContent = CarbonAILabelContent;
