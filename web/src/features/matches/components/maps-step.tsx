"use client";

import { useTranslations } from "next-intl";

import { Section } from "@/shared/ui/section";
import type { MatchTarget } from "../lib/match-target";
import { VetoError } from "./veto-error";
import { VetoMapBoard } from "./veto-map-board";
import { VetoTurn } from "./veto-turn";
import { VetoUndo } from "./veto-undo";

type Props = {
  target: MatchTarget;
  /** Passo feito, à vista para ver como terminou: só a grade. */
  done?: boolean;
};

/**
 * O passo Mapas (modo veto): o título com o desfazer ao lado, de quem é a
 * vez e o tempo, a grade do pool e o erro da última ação.
 */
export function MapsStep({ target, done = false }: Props) {
  const t = useTranslations("Match.steps");
  return (
    <Section
      title={t("maps")}
      action={done ? undefined : <VetoUndo target={target} />}
    >
      {!done && <VetoTurn />}
      <VetoMapBoard target={target} />
      {!done && <VetoError />}
    </Section>
  );
}
