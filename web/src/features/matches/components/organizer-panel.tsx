import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { MatchTarget } from "../lib/match-target";
import { ConfirmResultButton } from "./confirm-result-button";
import { ScoreForm } from "./score-form";
import { WalkoverForm } from "./walkover-form";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  maps: Schemas["Map"][];
  names: { a: string; b: string };
};

/**
 * Para organizer e owner, numa partida pronta ou jogada: aceitar o placar
 * enviado (resolve a contestação), lançar ou corrigir o placar e dar W.O.
 */
export function OrganizerPanel({ target, detail, maps, names }: Props) {
  const t = useTranslations("Match.organizer");
  const s = detail.summary;
  const r = s.result;
  const played = s.state === "completed";
  const initial =
    r && !r.walkover ? { a: r.a_wins ?? 0, b: r.b_wins ?? 0 } : undefined;

  return (
    <section className="flex flex-col gap-4 rounded-md border border-black/10 p-4 dark:border-white/15">
      <div className="flex flex-col gap-1">
        <h2 className="text-lg font-semibold">{t("title")}</h2>
        <p className="text-sm opacity-70">{t("hint")}</p>
      </div>
      {r && r.status !== "confirmed" && (
        <ConfirmResultButton
          target={target}
          label={r.status === "contested" ? t("accept") : t("confirm")}
        />
      )}
      <div className="flex flex-col gap-2">
        <h3 className="font-semibold">{played ? t("correct") : t("set")}</h3>
        {played && <p className="text-sm opacity-70">{t("correctHint")}</p>}
        <ScoreForm
          // Recomeça do placar que vale a cada atualização da página.
          key={`${r?.status}:${r?.a_wins}:${r?.b_wins}:${r?.walkover}`}
          target={target}
          mode="organizer"
          bestOf={s.best_of}
          names={names}
          maps={maps}
          initial={initial}
          submitLabel={t("save")}
        />
      </div>
      <WalkoverForm target={target} names={names} />
    </section>
  );
}
