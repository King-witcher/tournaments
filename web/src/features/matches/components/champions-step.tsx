"use client";

import { Column, Grid } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Section } from "@/shared/ui/section";
import { useVetoBoard } from "../hooks/use-veto-board";
import { currentGame } from "../lib/veto";
import styles from "./champions-step.module.scss";
import { DraftSummary } from "./draft-summary";
import { VetoChampionBoard } from "./veto-champion-board";
import { VetoError } from "./veto-error";
import { VetoTurnArea } from "./veto-turn-area";
import { VetoUndo } from "./veto-undo";
import { VetoVoteBoard } from "./veto-vote-board";

type Props = {
  rule: components["schemas"]["NoRepeatRule"] | null;
  /** Passo feito, à vista para ver como terminou: só o resumo. */
  done?: boolean;
};

/**
 * O passo Campeões: o título com o desfazer ao lado, de quem é a vez, a
 * grade e, ao lado dela (em cima no celular), o resumo do draft (bans e cada
 * partida, com a da vez em destaque), nas colunas da grade da página (11 e
 * 5 no lg, 10 e 6 do xlg para cima), e o erro da última ação. Na votação,
 * a do mapa da partida 1 vem antes (o draft depende dele); na janela, quem vê
 * escolhe para qual partida está escolhendo. Nos picks alternados, a partida é a
 * da vez. O capitão que escolhe pelo time diz para qual colega é o campeão, com o
 * próximo sem escolha já marcado; na sala sem nomes, os lugares são iguais, e vai
 * para o próximo sem perguntar.
 */
export function ChampionsStep({ rule, done = false }: Props) {
  const t = useTranslations("Match.veto.champions");
  const tGame = useTranslations("Match.games");
  const tSteps = useTranslations("Match.steps");
  const tMember = useTranslations("Member");
  const { veto } = useVetoBoard();
  const [chosenGame, setChosenGame] = useState<number | null>(null);
  const [chosenPlayer, setChosenPlayer] = useState<number | null>(null);
  if (!veto) return null;
  const voting = !done && veto.turn?.action === "vote";
  const slots = veto.picks;
  const games = [...new Set(slots.map((s) => s.game))];
  const game = games.includes(chosenGame ?? 0) ? chosenGame : games[0];
  const ofGame = slots.filter((s) => s.game === game);
  const slot = ofGame.find((s) => s.player === chosenPlayer) ?? ofGame[0];
  const teamPick = veto.turn?.action === "team_pick";
  const asksPlayer =
    !done && teamPick && !veto.nameless && veto.can_act.pick_for_team;
  const current =
    done || voting
      ? undefined
      : (slot?.game ?? veto.turn?.pick_for ?? undefined);
  const board = !done && !voting;

  return (
    <Section
      title={tSteps("champions")}
      action={done ? undefined : <VetoUndo />}
    >
      <VetoTurnArea turn={!done}>
        {voting && <VetoVoteBoard game={currentGame(veto)} />}
        {!done && teamPick && games.length > 1 && slot && (
          <RadioGroup
            legendText={t("forGame")}
            orientation="horizontal"
            value={String(slot.game)}
            options={games.map((g) => ({
              value: String(g),
              label: tGame("game", { number: g }),
            }))}
            onChange={(g) => setChosenGame(Number(g))}
          />
        )}
        {asksPlayer && slot && (
          <RadioGroup
            legendText={t("forPlayer")}
            orientation="horizontal"
            value={String(slot.player)}
            options={ofGame.map((s) => ({
              value: String(s.player),
              label: shownNickname(s.nickname ?? "", tMember("deletedAccount")),
            }))}
            onChange={(p) => setChosenPlayer(Number(p))}
          />
        )}
        <Grid>
          <Column
            sm={4}
            md={8}
            lg={board ? { span: 5, start: 12 } : 8}
            xlg={board ? { span: 6, start: 11 } : 8}
            className={styles.summary}
          >
            <DraftSummary current={current} />
          </Column>
          {board && (
            <Column
              sm={4}
              md={8}
              lg={{ span: 11, start: 1 }}
              xlg={{ span: 10, start: 1 }}
              className={styles.board}
            >
              <VetoChampionBoard rule={rule} slot={slot} />
            </Column>
          )}
        </Grid>
      </VetoTurnArea>
      {!done && <VetoError />}
    </Section>
  );
}
