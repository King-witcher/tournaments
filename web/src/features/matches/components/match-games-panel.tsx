import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { useSideNames } from "../hooks/use-side-names";
import { freeMaps } from "../lib/free-maps";
import type { MatchTarget } from "../lib/match-target";
import { GameCard } from "./game-card";
import { GameRow } from "./game-row";
import styles from "./match-games-panel.module.scss";
import { UpcomingGames } from "./upcoming-games";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  detail: Schemas["MatchDetail"];
  /** Os mapas ativos; cada formulário recebe só os que ainda cabem. */
  maps: Schemas["Map"][];
  /** Sem os passos da série em cima, a seção tem o título à vista. */
  titled: boolean;
};

/**
 * As partidas da série, na ordem: as enviadas numa linha compacta cada (com
 * o que dá para fazer nelas), a da vez num cartão com o envio e as que
 * faltam e o veto já decidiu, compactas. Some com a série que ainda não tem
 * lados.
 */
export function MatchGamesPanel({ target, detail, maps, titled }: Props) {
  const t = useTranslations("Match.games");
  const s = detail.summary;
  const names = useSideNames(detail);
  const next = detail.next_game;
  const open = s.state === "ready";
  if (!open && detail.games.length === 0) return null;

  const list = (
    <ol className={styles.list} aria-label={t("title")}>
      {detail.games.map((game) => (
        <GameRow
          key={game.number}
          target={target}
          game={game}
          names={names}
          maps={freeMaps(maps, detail.games, game.number)}
          canManage={detail.can_manage}
          last={game.number === detail.games.length}
        />
      ))}
      {next != null && (
        <GameCard
          target={target}
          detail={detail}
          number={next}
          names={names}
          maps={freeMaps(maps, detail.games, next)}
        />
      )}
      {open && <UpcomingGames after={next ?? detail.games.length} />}
    </ol>
  );
  return titled ? <Section title={t("title")}>{list}</Section> : list;
}
