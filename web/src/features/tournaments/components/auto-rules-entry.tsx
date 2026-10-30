import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { LocalDayTime } from "@/shared/ui/local-day-time";
import { AutoRulesBlock } from "./auto-rules-block";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * Inscrição e check-in, com o dia e a hora no fuso de quem vê ("hoje às
 * 21h"): até quando a inscrição vai e por onde ela entra, quando o check-in
 * abre e o que acontece com quem não o faz. Só antes de o torneio começar
 * (depois disso, nada aqui pede ação).
 */
export function AutoRulesEntry({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.entry");
  const formatMinutes = useFormatMinutes();
  const tdm = tournament.mode === "tdm";
  const { starts_at: startsAt, check_in_opens_at: opensAt } = tournament;
  const checkIn = tournament.check_in_minutes;
  const inviteOnly = tournament.admission === "invite_only";
  // O check-in no mesmo dia do início, que a frase antes já deu: só a hora.
  const when = (value: string, sameDayAs?: string | null) => () => (
    <LocalDayTime value={value} sameDayAs={sameDayAs} />
  );

  return (
    <AutoRulesBlock title={t(checkIn == null ? "titleNoCheckIn" : "title")}>
      <p>
        <Sentences>
          {[
            startsAt && t.rich("start", { when: when(startsAt) }),
            t("registration", { max: tournament.max_participants }),
            inviteOnly && t("inviteOnly"),
          ]}
        </Sentences>
      </p>
      <p>
        <Sentences>
          {checkIn == null
            ? [t(tdm ? "noCheckInTdm" : "noCheckIn")]
            : [
                tournament.status === "check_in"
                  ? t("checkInOpen")
                  : opensAt
                    ? t.rich("checkInAt", { when: when(opensAt, startsAt) })
                    : t("checkIn", { window: formatMinutes(checkIn) }),
                t(tdm ? "checkInDoTdm" : "checkInDo"),
                t("lateEntry"),
              ]}
        </Sentences>
      </p>
    </AutoRulesBlock>
  );
}
