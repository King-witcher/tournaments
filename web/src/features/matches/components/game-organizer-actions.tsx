"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { Stack } from "@/shared/ui/carbon/stack";
import { ErrorText } from "@/shared/ui/error-text";
import { Notice } from "@/shared/ui/notice";
import { useMarkMine } from "../hooks/use-live-match";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";
import { GameForm } from "./game-form";
import styles from "./game-organizer-actions.module.scss";
import { GameOrganizerMenu } from "./game-organizer-menu";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  maps: Schemas["Map"][];
  /** Quem vê já tem o "confirmar" de jogador nesta partida. */
  responds: boolean;
  /** É a última partida da série: a única que pode ser anulada. */
  last: boolean;
};

/**
 * Para organizer e owner, numa partida enviada, na linha das ações de
 * jogador: à vista, o que resolve (confirmar agora a pendente; na
 * contestada, confirmar a vitória enviada ou lançar outro resultado, que
 * não podem ficar escondidos) e, no ⋯, o raro (`GameOrganizerMenu`). Outro
 * resultado e a correção abrem o formulário no próprio cartão.
 */
export function GameOrganizerActions({
  target,
  game,
  names,
  maps,
  responds,
  last,
}: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const tForm = useTranslations("Match.form");
  const router = useRouter();
  const markMine = useMarkMine();
  const { run, pending, error } = useApiAction();
  const [editing, setEditing] = useState(false);
  const contested = game.status === "contested";
  const confirmable = contested || (game.status === "pending" && !responds);
  // O botão usado some (ao abrir ou fechar o formulário): o foco fica no
  // passo da partida.
  const close = () => {
    setEditing(false);
    focusIfLost(gameRowId(game.number));
  };
  const confirm = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params: gameParams(target, game.number) },
        ),
      () => {
        markMine();
        router.refresh();
        focusIfLost(gameRowId(game.number));
      },
      // Tela para trás (o jogador já confirmou): mostra o estado de agora.
      () => router.refresh(),
    );

  if (editing) {
    return (
      <Stack gap={5} className={styles.form}>
        {game.status === "confirmed" && (
          <Notice kind="info">{t("correctHint")}</Notice>
        )}
        <GameForm
          target={target}
          number={game.number}
          mode="organizer"
          names={names}
          maps={maps}
          initial={{
            map: game.map?.slug ?? "",
            a: game.a_frags ?? null,
            b: game.b_frags ?? null,
          }}
          submitLabel={tForm("save", { number: game.number })}
          onDone={close}
          onCancel={close}
          focusOnMount
        />
      </Stack>
    );
  }

  return (
    <div className={styles.actions}>
      {confirmable && (
        <Button size="sm" pending={pending} onClick={confirm}>
          {contested
            ? t("accept", { name: names[game.winner] })
            : t("confirmNow")}
        </Button>
      )}
      {contested && (
        <Button size="sm" kind="tertiary" onClick={() => setEditing(true)}>
          {t("other")}
        </Button>
      )}
      <GameOrganizerMenu
        target={target}
        game={game}
        last={last}
        onEdit={() => setEditing(true)}
      />
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </div>
  );
}
