import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable } from "@/shared/ui/data-table";
import { Link } from "@/shared/ui/link";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { useMatchLabels } from "../hooks/use-match-labels";
import { memberPath } from "../lib/paths";
import { eliminationOf } from "../lib/standings";
import { DepartureTag } from "./departure-tag";
import styles from "./standings-table.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  bracket: Schemas["TournamentBracket"];
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
};

/**
 * A classificação embaixo da chave, quando o torneio encerra: posição
 * (empatados dividem a faixa, 5º–8º), o jogador (link para a pessoa) e o
 * resultado (quem venceu o torneio, quem desistiu ou saiu, e a série que
 * eliminou cada um, com link). Antes disso, diz quando ela aparece.
 */
export function StandingsTable({ tenant, bracket, basePath }: Props) {
  const t = useTranslations("Bracket.standings");
  const tBracket = useTranslations("Bracket");
  const { code } = useMatchLabels(bracket.format.bracket === "single");
  const byUser = new Map(bracket.entrants.map((e) => [e.user_id, e]));
  const standings = bracket.status === "finished" ? bracket.standings : [];

  const rows = standings.map((s) => {
    const entrant = byUser.get(s.user_id);
    const nickname = entrant?.nickname ?? "";
    // O campeão pode ter perdido uma série (a grande final, vindo de baixo):
    // não foi eliminado.
    const champion = s.rank === 1 && s.tied === 1;
    const out = champion
      ? undefined
      : eliminationOf(bracket.matches, s.user_id);
    const r = out?.match.result;
    const score =
      r && out
        ? r.walkover
          ? tBracket("walkover")
          : out.side === "a"
            ? `${r.a_wins} × ${r.b_wins}`
            : `${r.b_wins} × ${r.a_wins}`
        : "";
    return {
      id: String(s.user_id),
      search: nickname,
      href: memberPath(tenant, s.user_id),
      cells: {
        rank:
          s.tied > 1
            ? t("range", { from: s.rank, to: s.rank + s.tied - 1 })
            : t("rank", { rank: s.rank }),
        player: (
          <span className={styles.player}>
            <Avatar src={entrant?.avatar_url} name={nickname} />
            <span className={styles.name}>{nickname}</span>
          </span>
        ),
        result: (
          <span className={styles.result}>
            {champion && (
              <StatusTag status="confirmed" size="sm">
                {t("champion")}
              </StatusTag>
            )}
            {entrant && (
              <DepartureTag withdrawn={entrant.withdrawn} left={entrant.left} />
            )}
            {out && (
              <span>
                {t.rich("eliminated", {
                  score,
                  match: () => (
                    <Link inline href={`${basePath}/${out.match.key}`}>
                      {code(out.match.key)}
                    </Link>
                  ),
                })}
              </span>
            )}
          </span>
        ),
      },
    };
  });

  return (
    <Section title={t("title")}>
      <DataTable
        label={t("title")}
        primaryColumn="player"
        columns={[
          { key: "rank", header: t("position") },
          { key: "player", header: t("player") },
          { key: "result", header: t("result") },
        ]}
        rows={rows}
        empty={{ text: t("empty") }}
      />
    </Section>
  );
}
