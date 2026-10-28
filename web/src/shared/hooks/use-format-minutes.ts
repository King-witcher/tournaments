import { useTranslations } from "next-intl";

/**
 * Formata minutos como duração curta: "10 min", "24 h", "1 h 30 min". "1440
 * min" não se lê de relance.
 */
export function useFormatMinutes() {
  const t = useTranslations("Common.duration");
  return (total: number) => {
    const hours = Math.floor(total / 60);
    const minutes = total % 60;
    if (hours === 0) return t("minutes", { minutes });
    if (minutes === 0) return t("hours", { hours });
    return t("hoursMinutes", { hours, minutes });
  };
}
