import { useTranslations } from "next-intl";

type Props = {
  /** Desistiu do torneio em andamento. */
  withdrawn: boolean;
  /** Saiu da comunidade (ou foi removido). */
  left: boolean;
};

/**
 * Marca discreta de quem não segue no torneio: "saiu" (da comunidade) ou
 * "desistiu". Nada para quem segue.
 */
export function DepartureMark({ withdrawn, left }: Props) {
  const t = useTranslations("Participants");
  if (!withdrawn && !left) return null;
  return (
    <span className="shrink-0 text-xs italic text-muted">
      {left ? t("left") : t("withdrawn")}
    </span>
  );
}
