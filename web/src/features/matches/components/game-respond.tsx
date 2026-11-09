"use client";

import { useTranslations } from "next-intl";
import { useId, useRef, useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { cn } from "@/shared/lib/cn";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { Stack } from "@/shared/ui/carbon/stack";
import { Checkbox } from "@/shared/ui/checkbox";
import { ErrorText } from "@/shared/ui/error-text";
import { TextArea } from "@/shared/ui/text-area";
import { useMarkMine } from "../hooks/use-live-match";
import {
  type GameValues,
  gameInput,
  gameParams,
  gameRowId,
  type MatchTarget,
} from "../lib/match-target";
import { validScore } from "../lib/score";
import { ContestProposalFields } from "./contest-proposal-fields";
import styles from "./game-respond.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  /** Os mapas que cabem na proposta (sem os já jogados na série). */
  maps: Schemas["Map"][];
  /** O mapa que o veto deu: a proposta fica presa a ele. */
  fixedMap?: Schemas["MapRef"];
  /** Do outro time, na partida pendente: também confirma. */
  canConfirm: boolean;
};

/** O comentário da contestação vai de 1 a 280 caracteres. */
const MAX_COMMENT = 280;

/**
 * Para quem pode responder a uma partida enviada: o outro time confirma ou
 * contesta; o colega de quem enviou, e todos depois de confirmada (enquanto a
 * contestação tardia vale), só contestam, com um comentário e, se marcar, o
 * resultado certo, para quem resolve aplicar num clique.
 */
export function GameRespond({
  target,
  game,
  names,
  maps,
  fixedMap,
  canConfirm,
}: Props) {
  const t = useTranslations("Match.respond");
  const router = useRouter();
  const hub = useRealtimeHub();
  const markMine = useMarkMine();
  const { run, pending, error } = useApiAction();
  const number = game.number;
  const sent: GameValues = {
    map: game.map?.slug ?? "",
    a: game.a_frags ?? null,
    b: game.b_frags ?? null,
  };
  const [contesting, setContesting] = useState(false);
  const [comment, setComment] = useState("");
  const [proposing, setProposing] = useState(false);
  const [proposal, setProposal] = useState(sent);
  const fieldId = useId();
  const contestButton = useRef<HTMLButtonElement>(null);
  const params = gameParams(target, number);
  // Como no `GameForm`: o mapa só vale se a lista o oferece, ou o do veto.
  const map =
    fixedMap?.slug ??
    (maps.some((m) => m.slug === proposal.map) ? proposal.map : "");
  const score = validScore(proposal.a, proposal.b);
  const proposed =
    proposing && map !== "" && score ? gameInput({ map, ...score }) : null;
  // A aba recarrega sozinha (`done`): o aviso da própria mudança não volta
  // pelo socket, que pediria uma segunda recarga.
  const headers = () => hub?.originHeaders();
  const done = () => {
    markMine();
    router.refresh();
    focusIfLost(gameRowId(number));
  };
  // A tela pode ter ficado para trás (o mapa já foi confirmado pela
  // organização, ou anulado): recarrega para mostrar o estado de agora.
  const stale = () => router.refresh();

  const confirm = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params, headers: headers() },
        ),
      done,
      stale,
    );
  const contest = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest",
          { params, body: { comment, proposal: proposed }, headers: headers() },
        ),
      done,
      stale,
    );
  // O botão que abriu some: o foco vai para o campo do comentário e, ao
  // cancelar, volta para o "Contestar".
  const open = () => {
    setContesting(true);
    requestAnimationFrame(() => document.getElementById(fieldId)?.focus());
  };
  const cancel = () => {
    setContesting(false);
    setProposing(false);
    setProposal(sent);
    requestAnimationFrame(() => contestButton.current?.focus());
  };

  return (
    <div className={cn(styles.respond, contesting && styles.open)}>
      {contesting ? (
        <Stack gap={5}>
          <TextArea
            id={fieldId}
            labelText={t("comment")}
            helperText={t("commentHint")}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            rows={2}
            enableCounter
            maxCount={MAX_COMMENT}
          />
          <Checkbox
            labelText={t("proposeResult")}
            checked={proposing}
            onChange={setProposing}
          />
          {proposing && (
            <ContestProposalFields
              value={proposal}
              onChange={setProposal}
              names={names}
              maps={maps}
              fixedMap={fixedMap}
              map={map}
            />
          )}
          <div className={styles.buttons}>
            <Button
              size="sm"
              pending={pending}
              disabled={proposing && !proposed}
              onClick={contest}
            >
              {t("submitContest")}
            </Button>
            <Button
              size="sm"
              kind="secondary"
              disabled={pending}
              onClick={cancel}
            >
              {t("cancel")}
            </Button>
          </div>
        </Stack>
      ) : (
        <div className={styles.buttons}>
          {canConfirm && (
            <Button size="sm" pending={pending} onClick={confirm}>
              {t("confirm")}
            </Button>
          )}
          <Button
            ref={contestButton}
            size="sm"
            kind="tertiary"
            disabled={pending}
            onClick={open}
          >
            {t("contest")}
          </Button>
        </div>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
