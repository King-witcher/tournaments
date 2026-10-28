import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { MatchTarget } from "../lib/match-target";
import { GameRow } from "./game-row";
import { NextGameRow } from "./next-game-row";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  maps: Schemas["Map"][];
  contestMinutes: number;
};

/**
 * Os mapas da série, um por linha, com o estado de cada um e o que dá para
 * fazer nele; por último, a linha do próximo mapa, enquanto a série aceita.
 */
export function MatchGamesPanel({
  target,
  detail,
  maps,
  contestMinutes,
}: Props) {
  const t = useTranslations("Match.games");
  const s = detail.summary;
  const names = { a: detail.a?.nickname ?? "", b: detail.b?.nickname ?? "" };
  const next = detail.next_game;
  const open = s.state === "ready";
  if (!open && detail.games.length === 0) return null;

  return (
    <section className="flex flex-col gap-1">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      {open && (
        <p className="text-sm opacity-70">
          {t("hint", { minutes: contestMinutes })}
        </p>
      )}
      {/* Divisória entre mapas: cada linha é uma unidade com estado e ações. */}
      <ol className="flex flex-col divide-y divide-black/10 dark:divide-white/15">
        {detail.games.map((game) => (
          <GameRow
            key={game.number}
            target={target}
            game={game}
            names={names}
            maps={maps}
            canManage={detail.can_manage}
          />
        ))}
        {next != null && (
          <NextGameRow
            target={target}
            number={next}
            names={names}
            maps={maps}
            plays={detail.viewer_side != null}
            enters={detail.can_manage && detail.viewer_side == null}
          />
        )}
      </ol>
    </section>
  );
}
