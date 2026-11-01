import { Accordion } from "@carbon/react";

import type { components } from "@/shared/api/schema";
import type { MatchTarget } from "../lib/match-target";
import { CorrectionNote } from "./correction-note";
import { MatchHistory } from "./match-history";
import { OrganizerPanel } from "./organizer-panel";
import { PendingMatches } from "./pending-matches";
import { VetoHistory } from "./veto-history";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  /** A chave do torneio, ou `null` se a API falhou. */
  bracket: Schemas["TournamentBracket"] | null;
  names: { a: string; b: string };
};

/**
 * O fim da página da série: o aviso de correção recusada (organização, série
 * decidida), as outras séries que esperam quem vê e, fechados, o W.O. da
 * organização e o histórico (o da série e o do veto).
 */
export function MatchAfter({ target, detail, bracket, names }: Props) {
  const s = detail.summary;
  const r = s.result;
  const playable = s.state === "ready" || s.state === "completed";
  const basePath = `/${target.tenant}/${target.slug}/matches`;
  return (
    <>
      {detail.can_manage && s.state === "completed" && bracket && (
        <CorrectionNote
          bracket={bracket}
          matchKey={s.key}
          basePath={basePath}
        />
      )}
      {bracket && (
        // O que mais espera quem vê, para seguir depois de agir nesta.
        <PendingMatches bracket={bracket} basePath={basePath} exclude={s.key} />
      )}
      <Accordion>
        {detail.can_manage && playable && (
          // A chave muda a cada resultado: o bloco fecha depois de salvar.
          <OrganizerPanel
            key={`${r?.winner}:${r?.walkover}:${r?.a_wins}:${r?.b_wins}`}
            target={target}
            names={names}
          />
        )}
        <MatchHistory detail={detail} />
        {detail.veto && <VetoHistory veto={detail.veto} names={names} />}
      </Accordion>
    </>
  );
}
