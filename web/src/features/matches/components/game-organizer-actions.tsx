"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button, type ButtonKind } from "@/shared/ui/button";
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
 * jogador. Na contestada, tudo à vista: com resultado proposto, aceitar
 * (o primário), manter e lançar outro; sem ele, lançar outro é o primário;
 * anular fica no fim (`GameOrganizerMenu`). Nas outras, confirmar agora a
 * pendente e, no ⋯, o raro. Outro resultado e a correção abrem o formulário no
 * próprio cartão.
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
  const tRespond = useTranslations("Match.respond");
  const tForm = useTranslations("Match.form");
  const router = useRouter();
  const markMine = useMarkMine();
  const { runAs, buttonFor, pending, error } = useApiAction();
  const [editing, setEditing] = useState(false);
  const contested = game.status === "contested";
  const proposal = game.contest_proposal;
  // O botão usado some (ao abrir ou fechar o formulário): o foco fica no
  // passo da partida.
  const close = () => {
    setEditing(false);
    focusIfLost(gameRowId(game.number));
  };
  const done = () => {
    markMine();
    router.refresh();
    focusIfLost(gameRowId(game.number));
  };
  // Tela para trás (o jogador já confirmou): mostra o estado de agora.
  const stale = () => router.refresh();
  const params = gameParams(target, game.number);
  const confirm = () =>
    runAs(
      "confirm",
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params },
        ),
      done,
      stale,
    );
  const acceptProposal = (p: Schemas["ContestProposal"]) =>
    runAs(
      "accept",
      () =>
        api.PUT(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}",
          {
            params,
            body: {
              game: {
                map: p.map.slug,
                a_frags: p.a_frags,
                b_frags: p.b_frags,
              },
            },
          },
        ),
      done,
      stale,
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

  const other = (kind: ButtonKind) => (
    <Button
      size="sm"
      kind={kind}
      disabled={pending}
      onClick={() => setEditing(true)}
    >
      {t("other")}
    </Button>
  );

  return (
    <div className={styles.actions}>
      {proposal && (
        <Button
          size="sm"
          {...buttonFor("accept")}
          onClick={() => acceptProposal(proposal)}
        >
          {tRespond("acceptContest")}
        </Button>
      )}
      {contested && !proposal && other("primary")}
      {contested && (
        <Button
          size="sm"
          kind="tertiary"
          {...buttonFor("confirm")}
          onClick={confirm}
        >
          {t("keep")}
        </Button>
      )}
      {contested && proposal && other("tertiary")}
      {game.status === "pending" && !responds && (
        <Button size="sm" {...buttonFor("confirm")} onClick={confirm}>
          {t("confirmNow")}
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
