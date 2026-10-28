"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import type { GameValues } from "../lib/match-target";
import { GameRow } from "./game-row";

type Props = {
  games: GameValues[];
  onChange: (games: GameValues[]) => void;
  maps: components["schemas"]["Map"][];
  names: { a: string; b: string };
  /** Quantas linhas cabem (os mapas jogados pelo placar). */
  max: number;
};

/** Detalhe opcional por mapa: uma linha por mapa, na ordem jogada. */
export function GamesEditor({ games, onChange, maps, names, max }: Props) {
  const t = useTranslations("Match.report");
  const set = (i: number, game: GameValues) =>
    onChange(games.map((g, j) => (j === i ? game : g)));

  return (
    <fieldset className="flex flex-col gap-2">
      <legend className="text-sm font-medium">{t("detail")}</legend>
      <p className="text-xs opacity-70">{t("detailHint")}</p>
      {games.map((game, i) => (
        <GameRow
          // A linha é a posição do mapa na série.
          // biome-ignore lint/suspicious/noArrayIndexKey: a posição é o próprio dado.
          key={i}
          number={i + 1}
          game={game}
          onChange={(next) => set(i, next)}
          onRemove={() => onChange(games.filter((_, j) => j !== i))}
          maps={maps}
          names={names}
        />
      ))}
      <div>
        <Button
          size="sm"
          variant="outline"
          disabled={games.length >= max}
          onClick={() => onChange([...games, { map: "", a: "", b: "" }])}
        >
          {t("addGame")}
        </Button>
      </div>
    </fieldset>
  );
}
