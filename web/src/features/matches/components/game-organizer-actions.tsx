"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { cn } from "@/shared/lib/cn";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { Notice } from "@/shared/ui/notice";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { TextInput } from "@/shared/ui/text-input";
import { gameParams, gameRowId, type MatchTarget } from "../lib/match-target";
import { GameForm } from "./game-form";
import styles from "./game-organizer-actions.module.scss";

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
  /** Na linha dos botões de jogador do cartão (o formulário ocupa a largura toda). */
  className?: string;
};

/**
 * Para organizer e owner, numa partida enviada, na mesma linha das ações de
 * jogador: à vista, o que resolve (confirmar agora a pendente; na
 * contestada, confirmar a vitória enviada ou lançar outro resultado, que
 * não podem ficar escondidos); no ⋯, o que é raro: lançar outro resultado
 * ou corrigir, com o formulário no próprio cartão, e anular a última sem
 * confirmar para ser jogada de novo (Modal com a nota do histórico).
 */
export function GameOrganizerActions({
  target,
  game,
  names,
  maps,
  responds,
  last,
  className,
}: Props) {
  const t = useTranslations("Match.gameOrganizer");
  const tForm = useTranslations("Match.form");
  const router = useRouter();
  const confirming = useApiAction();
  const replaying = useApiAction();
  const [editing, setEditing] = useState(false);
  const [note, setNote] = useState("");
  const contested = game.status === "contested";
  const confirmed = game.status === "confirmed";
  const confirmable = contested || (game.status === "pending" && !responds);
  const params = gameParams(target, game.number);
  // O botão usado some (ao abrir ou fechar o formulário, depois de agir): o
  // foco fica no passo da partida.
  const done = () => {
    router.refresh();
    focusIfLost(gameRowId(game.number));
  };
  // Tela para trás (o jogador já confirmou, o prazo passou): mostra o de agora.
  const stale = () => router.refresh();
  const close = () => {
    setEditing(false);
    focusIfLost(gameRowId(game.number));
  };
  const confirm = () =>
    confirming.run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params },
        ),
      done,
      stale,
    );
  const replay = () =>
    replaying.run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/replay",
          { params, body: { comment: note.trim() || null } },
        ),
      () => {
        setNote("");
        done();
      },
      stale,
    );

  if (editing) {
    return (
      <Stack gap={5} className={cn(styles.form, className)}>
        {confirmed && <Notice kind="info">{t("correctHint")}</Notice>}
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

  const rare: OverflowAction[] = [];
  if (!contested)
    rare.push({
      label: confirmed ? t("correct") : t("other"),
      onSelect: () => setEditing(true),
    });
  if (last && !confirmed)
    rare.push({
      label: t("replay"),
      destructive: true,
      onSelect: replay,
      confirm: {
        title: t("replaySubmit", { number: game.number }),
        question: t("replayQuestion", { number: game.number }),
        extra: (
          <TextInput
            labelText={t("note")}
            value={note}
            onChange={(e) => setNote(e.target.value)}
            maxLength={280}
          />
        ),
      },
    });
  const error = confirming.error ?? replaying.error;

  return (
    <div className={cn(styles.actions, className)}>
      {confirmable && (
        <Button size="sm" pending={confirming.pending} onClick={confirm}>
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
      {rare.length > 0 && (
        <OverflowActions
          items={rare}
          size="sm"
          pending={replaying.pending}
          label={t("more", { number: game.number })}
        />
      )}
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </div>
  );
}
