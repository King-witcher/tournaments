"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { gameRowId, type MatchTarget } from "../lib/match-target";
import styles from "./game-accept-contest.module.scss";
import { GameForm } from "./game-form";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  maps: Schemas["Map"][];
  /** O mapa que o veto deu: fica preso a ele, sem a escolha. */
  fixedMap?: Schemas["MapRef"];
};

/**
 * Para o capitão do time adversário de quem contestou: aceita a contestação sem
 * esperar a organização. O comentário não traz resultado, então o botão abre o
 * formulário com o placar enviado, para ele corrigir.
 */
export function GameAcceptContest({
  target,
  game,
  names,
  maps,
  fixedMap,
}: Props) {
  const t = useTranslations("Match.respond");
  const [accepting, setAccepting] = useState(false);
  // O botão usado some (ao abrir ou fechar o formulário): o foco fica no
  // passo da partida.
  const close = () => {
    setAccepting(false);
    focusIfLost(gameRowId(game.number));
  };

  if (!accepting) {
    return (
      <Button size="sm" onClick={() => setAccepting(true)}>
        {t("acceptContest")}
      </Button>
    );
  }
  return (
    <Stack gap={5} className={styles.form}>
      <Notice kind="info">{t("acceptHint")}</Notice>
      <GameForm
        target={target}
        number={game.number}
        mode="accept"
        names={names}
        maps={maps}
        fixedMap={fixedMap}
        initial={{
          map: game.map?.slug ?? "",
          a: game.a_frags ?? null,
          b: game.b_frags ?? null,
        }}
        submitLabel={t("acceptSubmit")}
        onDone={close}
        onCancel={close}
        focusOnMount
      />
    </Stack>
  );
}
