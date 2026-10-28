import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { MatchTarget } from "../lib/match-target";
import { DeadlineCountdown } from "./deadline-countdown";
import { OrganizerPanel } from "./organizer-panel";
import { RespondPanel } from "./respond-panel";
import { ScoreForm } from "./score-form";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  slug: string;
  detail: Schemas["MatchDetail"];
  maps: Schemas["Map"][];
  contestMinutes: number;
};

/**
 * O que quem vê pode fazer na partida, conforme o que ela espera dele
 * (enviar, responder, esperar) e, para a organização, o painel dela.
 */
export function MatchActions({
  tenant,
  slug,
  detail,
  maps,
  contestMinutes,
}: Props) {
  const t = useTranslations("Match");
  const s = detail.summary;
  const r = s.result;
  const target: MatchTarget = { tenant, slug, matchKey: s.key };
  const names = { a: detail.a?.nickname ?? "", b: detail.b?.nickname ?? "" };
  const deadline = r?.confirm_by;

  return (
    <>
      {s.task === "report" && (
        <section className="flex flex-col gap-3">
          <h2 className="text-xl font-semibold">{t("report.title")}</h2>
          <p className="text-sm opacity-70">
            {t("report.hint", { minutes: contestMinutes })}
          </p>
          <ScoreForm
            target={target}
            mode="report"
            bestOf={s.best_of}
            names={names}
            maps={maps}
            submitLabel={t("report.submit")}
          />
        </section>
      )}
      {s.task === "respond" && r?.reported_side && deadline && (
        <RespondPanel
          target={target}
          reporter={names[r.reported_side]}
          deadline={deadline}
        />
      )}
      {s.task === "await_opponent" && deadline && (
        <p className="text-sm">
          {t.rich("waiting.opponent", {
            time: () => <DeadlineCountdown until={deadline} />,
          })}
        </p>
      )}
      {s.task === "await_organizer" && (
        <p className="text-sm">{t("waiting.organizer")}</p>
      )}
      {detail.can_manage &&
        (s.state === "ready" || s.state === "completed") && (
          <OrganizerPanel
            target={target}
            detail={detail}
            maps={maps}
            names={names}
          />
        )}
    </>
  );
}
