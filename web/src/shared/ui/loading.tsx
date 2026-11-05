import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import { Loading as CarbonLoading } from "./carbon/loading";

// `id` está obsoleto no Carbon.
type Props = Omit<ComponentProps<typeof CarbonLoading>, "id">;

/**
 * Indicador de carregamento grande (`Loading` do Carbon), para uma espera sem
 * forma conhecida que ocupa a página ou um bloco. Com `withOverlay` (o padrão
 * do Carbon), cobre a tela e impede o uso até terminar; `small` tem o tamanho
 * de um bloco; `active={false}` para o giro antes de sair. Dentro de um
 * controle ou ao lado de um texto é o `InlineLoading` (no botão pendente, o
 * `Spinner`); conteúdo com forma conhecida carrega com os esqueletos de
 * `skeleton.tsx`, para a página não pular.
 *
 * O `description`, lido pelo leitor de tela, é "Carregando…" quando não vem.
 */
export function Loading({ description, ...props }: Props) {
  const t = useTranslations("Common");
  return <CarbonLoading description={description ?? t("loading")} {...props} />;
}
