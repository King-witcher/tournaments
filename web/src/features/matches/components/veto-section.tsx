import { useTranslations } from "next-intl";

import { LiveUpdates } from "@/features/realtime/components/live-updates";
import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { useSideNames } from "../hooks/use-side-names";
import { type MatchTarget, VETO_ID } from "../lib/match-target";
import { currentGame } from "../lib/veto";
import styles from "./veto-board.module.scss";
import { VetoChampionBoard } from "./veto-champion-board";
import { VetoGames } from "./veto-games";
import { VetoMapBoard } from "./veto-map-board";
import { VetoTurn } from "./veto-turn";
import { VetoUndo } from "./veto-undo";
import { VetoVoteBoard } from "./veto-vote-board";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  /** A regra de não repetição do torneio (para explicar o que ela tira). */
  rule: Schemas["NoRepeatScope"] | null;
};

/**
 * O veto da partida, entre ela ficar pronta e o envio de cada mapa: de quem
 * é a vez e o tempo, o veto de mapas (ou a votação do jogo), os campeões do
 * mapa em andamento, desfazer e o que já foi decidido. Enquanto o veto anda,
 * a página recarrega com o aviso mais rápido (cada passo tem 30 s).
 */
export function VetoSection({ target, detail, rule }: Props) {
  const t = useTranslations("Match.veto");
  const names = useSideNames(detail);
  const veto = detail.veto;
  if (!veto) return null;
  const s = detail.summary;
  const live = s.state === "ready";
  const mine = detail.viewer_side ?? null;
  const game = currentGame(veto);
  const current = veto.games.find((g) => g.game === game);
  const begun = veto.status !== "waiting_ready";

  return (
    <Section title={t("title")} description={t(`mode.${veto.map_mode}`)}>
      {live && veto.status !== "done" && (
        <LiveUpdates
          tenant={target.tenant}
          tournament={target.slug}
          match={target.matchKey}
          fast
        />
      )}
      {/* tabIndex -1: recebe o foco depois de um passo cujo botão some. */}
      <div id={VETO_ID} tabIndex={-1} className={styles.board}>
        {live && (
          <VetoTurn
            target={target}
            veto={veto}
            names={names}
            mine={mine}
            canManage={detail.can_manage}
          />
        )}
        {live && (
          <VetoUndo
            target={target}
            veto={veto}
            names={names}
            mine={mine}
            canManage={detail.can_manage}
          />
        )}
        {begun && veto.map_mode === "veto" && (
          <VetoMapBoard
            target={target}
            veto={veto}
            bestOf={s.best_of}
            names={names}
          />
        )}
        {begun && veto.map_mode === "vote" && current && !current.map && (
          <VetoVoteBoard
            target={target}
            veto={veto}
            game={current}
            names={names}
          />
        )}
        {live && begun && veto.champion_phase && (
          <VetoChampionBoard
            target={target}
            veto={veto}
            game={game}
            names={names}
            mine={mine}
            rule={rule}
          />
        )}
        <VetoGames veto={veto} names={names} />
      </div>
    </Section>
  );
}
