"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
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
 * esperar a organização. Com o resultado proposto, um clique o aplica; sem ele,
 * o botão abre o formulário com o placar enviado, para o capitão corrigir.
 */
export function GameAcceptContest(props: Props) {
  const proposal = props.game.contest_proposal;
  return proposal ? (
    <AcceptProposal {...props} proposal={proposal} />
  ) : (
    <AcceptWithScore {...props} />
  );
}

function AcceptProposal({
  target,
  game,
  proposal,
}: Props & { proposal: Schemas["ContestProposal"] }) {
  const t = useTranslations("Match.respond");
  const router = useRouter();
  const hub = useRealtimeHub();
  const markMine = useMarkMine();
  const { run, pending, error } = useApiAction();
  const accept = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest/accept",
          {
            params: gameParams(target, game.number),
            body: {
              map: proposal.map.slug,
              a_frags: proposal.a_frags,
              b_frags: proposal.b_frags,
            },
            headers: hub?.originHeaders(),
          },
        ),
      () => {
        markMine();
        router.refresh();
        focusIfLost(gameRowId(game.number));
      },
      // A tela pode ter ficado para trás: mostra o estado de agora.
      () => router.refresh(),
    );

  return (
    <>
      <Button size="sm" pending={pending} onClick={accept}>
        {t("acceptContest")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </>
  );
}

function AcceptWithScore({ target, game, names, maps, fixedMap }: Props) {
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
