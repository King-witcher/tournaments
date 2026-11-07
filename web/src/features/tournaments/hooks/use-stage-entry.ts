import { useTranslations } from "next-intl";

/**
 * A regra de entrada de uma fase, da segunda em diante, em frases: quem entra
 * da fase anterior (todos ou os N melhores) e, se for o caso, o check-in antes
 * da fase. `number` é a fase (a partir de 1).
 */
export function useStageEntry() {
  const t = useTranslations("Tournament.setup.entry");
  return (
    advance: number | null | undefined,
    checkIn: boolean | undefined,
    number: number,
  ): string[] => [
    advance != null
      ? t("top", { count: advance, previous: number - 1 })
      : t("all", { previous: number - 1 }),
    ...(checkIn ? [t("checkIn")] : []),
  ];
}
