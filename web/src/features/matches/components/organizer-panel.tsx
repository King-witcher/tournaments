import { useTranslations } from "next-intl";

import { Disclosure } from "@/shared/ui/disclosure";
import type { MatchTarget } from "../lib/match-target";
import { WalkoverForm } from "./walkover-form";

type Props = {
  target: MatchTarget;
  names: { a: string; b: string };
  /** Muda a cada resultado: o bloco fecha depois de salvar. */
  resultKey: string;
};

/**
 * Para organizer e owner, numa partida pronta ou decidida: o W.O. na série
 * inteira, fechado atrás de um botão porque é exceção. Lançar, resolver e
 * corrigir mapas fica na linha de cada mapa.
 */
export function OrganizerPanel({ target, names, resultKey }: Props) {
  const t = useTranslations("Match.organizer");

  return (
    <section className="flex flex-col gap-3 rounded-md border border-black/10 p-4 dark:border-white/15">
      <div className="flex flex-col gap-1">
        <h2 className="text-lg font-semibold">{t("title")}</h2>
        <p className="text-sm text-muted">{t("hint")}</p>
      </div>
      <Disclosure key={resultKey} summary={t("walkoverTitle")}>
        <WalkoverForm target={target} names={names} />
      </Disclosure>
    </section>
  );
}
