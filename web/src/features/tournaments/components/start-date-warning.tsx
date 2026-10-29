import { useTranslations } from "next-intl";

import { Notice } from "@/shared/ui/notice";
import { startTiming } from "../lib/form-values";

type Props = {
  /** Início em ISO (UTC), ou `null` sem data. */
  startsAt: string | null;
  /** Janela de check-in em minutos, ou `null` sem check-in. */
  checkInMinutes: number | null;
  /** Referência para comparar com a data. */
  now: Date;
};

/**
 * Aviso (não bloqueia) de que a data de início não deixa tempo para os
 * jogadores se inscreverem e fazerem check-in: já passou, ou está mais perto
 * do que a janela de check-in (que abriria na hora). Nada sem data ou com
 * tempo de sobra. Único aviso do tipo: formulário, revisão e publicação.
 */
export function StartDateWarning({ startsAt, checkInMinutes, now }: Props) {
  const t = useTranslations("TournamentForm.startWarning");
  const timing = startTiming(startsAt, checkInMinutes, now);
  if (timing === null) return null;
  return (
    <Notice kind="warning">
      {timing === "past"
        ? t("past")
        : t("window", { minutes: checkInMinutes ?? 0 })}
    </Notice>
  );
}
