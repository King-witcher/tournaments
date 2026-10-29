import { useTranslations } from "next-intl";

/**
 * Formata segundos do veto como duração curta: "30 s", "1 min", "1 min 30 s".
 * A partir de um minuto, em minutos, como as janelas em horas do resto do
 * formulário ("120 s" não se lê de relance).
 */
export function useFormatSeconds() {
  const t = useTranslations("TournamentForm.duration");
  return (total: number) => {
    const minutes = Math.floor(total / 60);
    const seconds = total % 60;
    if (minutes === 0) return t("seconds", { seconds });
    if (seconds === 0) return t("minutes", { minutes });
    return t("minutesSeconds", { minutes, seconds });
  };
}
