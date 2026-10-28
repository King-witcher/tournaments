import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Disclosure } from "@/shared/ui/disclosure";
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
 * enviado (resolve a contestação) à vista e, fechados atrás de um botão,
 * lançar ou corrigir o placar e dar W.O. Fechados porque são exceção, e
 * porque o organizador que joga a partida já vê o envio de jogador acima.
 */
export function OrganizerPanel({ target, detail, maps, names }: Props) {
  const t = useTranslations("Match.organizer");
  const s = detail.summary;
  const r = s.result;
  const played = s.state === "completed";
  const reported = r != null && r.status !== "confirmed";
  // Muda a cada resultado: o formulário recomeça do placar que vale e os
  // blocos fecham depois de salvar.
  const resultKey = `${r?.status}:${r?.a_wins}:${r?.b_wins}:${r?.walkover}:${r?.winner}`;
  const initial =
    r && !r.walkover ? { a: r.a_wins ?? 0, b: r.b_wins ?? 0 } : undefined;

  return (
    <section className="flex flex-col gap-3 rounded-md border border-black/10 p-4 dark:border-white/15">
      <div className="flex flex-col gap-1">
        <h2 className="text-lg font-semibold">{t("title")}</h2>
        <p className="text-sm opacity-70">{t("hint")}</p>
      </div>
      {reported && (
        <ConfirmResultButton
          target={target}
          label={r.status === "contested" ? t("accept") : t("confirm")}
        />
      )}
      <Disclosure
        key={`score:${resultKey}`}
        summary={played ? t("correct") : reported ? t("setOther") : t("set")}
      >
        {played && <p className="text-sm opacity-70">{t("correctHint")}</p>}
        <ScoreForm
          target={target}
          mode="organizer"
          bestOf={s.best_of}
          names={names}
          maps={maps}
          initial={initial}
          submitLabel={t("save")}
        />
      </Disclosure>
      <Disclosure key={`walkover:${resultKey}`} summary={t("walkoverTitle")}>
        <WalkoverForm target={target} names={names} />
      </Disclosure>
    </section>
  );
}
