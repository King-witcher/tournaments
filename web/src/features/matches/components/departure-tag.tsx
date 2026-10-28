import { useTranslations } from "next-intl";

import { StatusTag } from "@/shared/ui/status-tag";

type Props = {
  /** Desistiu do torneio em andamento. */
  withdrawn: boolean;
  /** Saiu da comunidade (ou foi removido). */
  left: boolean;
};

/**
 * Quem não segue no torneio, ao lado do apelido: "saiu" (da comunidade) ou
 * "desistiu", na cor de cancelado. Nada para quem segue.
 */
export function DepartureTag({ withdrawn, left }: Props) {
  const t = useTranslations("Bracket.departure");
  if (!withdrawn && !left) return null;
  return (
    <StatusTag status="cancelled" size="sm">
      {left ? t("left") : t("withdrawn")}
    </StatusTag>
  );
}
