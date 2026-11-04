import type { ReactNode } from "react";

import { LiveUpdates } from "@/features/realtime/components/live-updates";
import type { components } from "@/shared/api/schema";
import { useSideNames } from "../hooks/use-side-names";
import { freeMaps } from "../lib/free-maps";
import { gameStep, type StepKey } from "../lib/match-steps";
import type { MatchTarget } from "../lib/match-target";
import { GameCard } from "./game-card";
import { GamePlayed } from "./game-played";
import { GameStep } from "./game-step";
import { GameVeto } from "./game-veto";
import { LiveMatchProvider } from "./live-match-provider";
import { MatchSteps } from "./match-steps";
import { VetoUndo } from "./veto-undo";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  tournament: Schemas["Tournament"];
  /** Os mapas ativos (os formulários de envio recebem os que cabem). */
  maps: Schemas["Map"][];
  viewerId: number | null;
};

/**
 * O meio da página da série, o que anda ao vivo: os passos, com a série
 * guardada no `LiveMatchProvider` para as ações do veto aparecerem na hora,
 * e cada partida que abre (as enviadas e a da vez), desenhada aqui (os
 * screenshots vêm do servidor). Enquanto o veto anda, a página recarrega com
 * o aviso mais rápido (cada passo tem 30 s).
 */
export function MatchLive({
  target,
  detail,
  tournament,
  maps,
  viewerId,
}: Props) {
  const s = detail.summary;
  const names = useSideNames(detail);
  const games: Partial<Record<StepKey, ReactNode>> = {};
  for (const game of detail.games) {
    games[gameStep(game.number)] = (
      <GameStep number={game.number}>
        <GamePlayed
          target={target}
          game={game}
          names={names}
          maps={freeMaps(maps, detail.games, game.number)}
          fixedMap={
            detail.veto?.games.find((g) => g.game === game.number)?.map ??
            undefined
          }
          canManage={detail.can_manage}
          last={game.number === detail.games.length}
        />
      </GameStep>
    );
  }
  const next = detail.next_game;
  if (next != null && !games[gameStep(next)]) {
    games[gameStep(next)] = (
      <GameStep number={next} action={<VetoUndo />}>
        <GameVeto number={next} />
        <GameCard
          target={target}
          detail={detail}
          number={next}
          names={names}
          maps={freeMaps(maps, detail.games, next)}
        />
      </GameStep>
    );
  }

  return (
    <>
      {s.state === "ready" && detail.veto && detail.veto.status !== "done" && (
        <LiveUpdates
          tenant={target.tenant}
          tournament={target.slug}
          match={s.key}
          fast
        />
      )}
      <LiveMatchProvider
        target={target}
        initial={detail}
        viewerId={viewerId}
        bansPerSeries={tournament.champion_rules.bans_per_series}
        vetoOrder={tournament.veto.order}
      >
        <MatchSteps
          rule={tournament.champion_rules.no_repeat ?? null}
          games={games}
        />
      </LiveMatchProvider>
    </>
  );
}
