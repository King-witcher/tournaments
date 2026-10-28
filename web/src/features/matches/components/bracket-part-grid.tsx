import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";
import type { Round } from "../lib/bracket-layout";
import { MatchCard } from "./match-card";

type Schemas = components["schemas"];

type Props = {
  /** Título da parte (só na dupla). */
  title?: string;
  part: Schemas["BracketPart"];
  rounds: Round[];
  single: boolean;
  entrants: Map<number, Schemas["BracketEntrant"]>;
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
};

/**
 * Uma parte da chave: uma coluna por rodada, com as partidas distribuídas na
 * altura para lembrar o chaveamento. Rola na horizontal em tela pequena.
 */
export function BracketPartGrid({
  title,
  part,
  rounds,
  single,
  entrants,
  basePath,
}: Props) {
  const { roundTitle } = useMatchLabels(single);
  const lastRound = rounds[rounds.length - 1]?.round ?? 1;

  return (
    <div className="flex flex-col gap-2">
      {title && <h3 className="font-semibold">{title}</h3>}
      <div className="overflow-x-auto pb-2">
        <div className="flex w-max gap-4">
          {rounds.map(({ round, matches }) => (
            <div key={round} className="flex w-56 flex-col gap-2">
              <h4 className="text-xs font-medium uppercase tracking-wide opacity-60">
                {roundTitle(part, round, lastRound)}
              </h4>
              <ol className="flex flex-1 flex-col justify-around gap-3">
                {matches.map((m) => (
                  <li key={m.key}>
                    <MatchCard
                      match={m}
                      single={single}
                      entrants={entrants}
                      href={`${basePath}/${m.key}`}
                    />
                  </li>
                ))}
              </ol>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
