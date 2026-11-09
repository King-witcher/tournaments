import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDayTime } from "@/shared/ui/local-day-time";
import { AutoRulesBlock } from "./auto-rules-block";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * Inscrição e check-in, com o dia e a hora no fuso de quem vê ("hoje às
 * 21h"): até quando a inscrição vai e por onde ela entra, o check-in e o que
 * acontece com quem não o faz. Só antes de o torneio começar (depois disso,
 * nada aqui pede ação).
 */
export function AutoRulesEntry({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.entry");
  const tdm = tournament.mode === "tdm";
  const { starts_at: startsAt, check_in: checkIn } = tournament;
  const inviteOnly = tournament.admission === "invite_only";

  return (
    <AutoRulesBlock title={t(checkIn ? "title" : "titleNoCheckIn")}>
      <p>
        <Sentences>
          {[
            startsAt &&
              t.rich("start", {
                when: () => <LocalDayTime value={startsAt} />,
              }),
            t("registration", { max: tournament.max_participants }),
            inviteOnly && t("inviteOnly"),
          ]}
        </Sentences>
      </p>
      <p>
        <Sentences>
          {checkIn
            ? [
                t(tournament.status === "check_in" ? "checkInOpen" : "checkIn"),
                t(tdm ? "checkInDoTdm" : "checkInDo"),
                t("lateEntry"),
              ]
            : [t(tdm ? "noCheckInTdm" : "noCheckIn")]}
        </Sentences>
      </p>
    </AutoRulesBlock>
  );
}
