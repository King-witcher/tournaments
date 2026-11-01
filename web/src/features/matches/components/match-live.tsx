import { LiveUpdates } from "@/features/realtime/components/live-updates";
import type { components } from "@/shared/api/schema";
import { seriesSteps } from "../lib/match-steps";
import type { MatchTarget } from "../lib/match-target";
import { LiveMatchProvider } from "./live-match-provider";
import { MatchGamesPanel } from "./match-games-panel";
import { MatchSteps } from "./match-steps";

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
 * O meio da página da série, o que anda ao vivo: os passos (com veto no
 * torneio) e as partidas, com a série guardada no \`LiveMatchProvider\` para
 * as ações do veto aparecerem na hora. Enquanto o veto anda, a página
 * recarrega com o aviso mais rápido (cada passo tem 30 s).
 */
export function MatchLive({
  target,
  detail,
  tournament,
  maps,
  viewerId,
}: Props) {
  const s = detail.summary;
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
        initial={detail}
        viewerId={viewerId}
        bansPerSeries={tournament.champion_rules.bans_per_series}
        vetoOrder={tournament.veto.order}
      >
        <MatchSteps
          target={target}
          rule={tournament.champion_rules.no_repeat ?? null}
          games={
            <MatchGamesPanel
              target={target}
              detail={detail}
              maps={maps}
              titled={!seriesSteps(detail)}
            />
          }
        />
      </LiveMatchProvider>
    </>
  );
}
