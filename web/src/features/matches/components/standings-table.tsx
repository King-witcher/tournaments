import { useTranslations } from "next-intl";
import { memberPath } from "@/features/members/lib/paths";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable } from "@/shared/ui/data-table";
import { Link } from "@/shared/ui/link";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { useMatchLabels } from "../hooks/use-match-labels";
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
 * (empatados dividem a faixa, 5º–8º), o time (no duelo, o jogador, com link
 * para a pessoa) e o
 * resultado (quem venceu o torneio, quem desistiu ou saiu, e a série que
 * eliminou cada um, com link). Antes disso, diz quando ela aparece.
 */
export function StandingsTable({ tenant, bracket, basePath }: Props) {
  const t = useTranslations("Bracket.standings");
  const tBracket = useTranslations("Bracket");
  const { code } = useMatchLabels(isSingle(bracket.format));
  const teamName = useTeamName();
  const byId = new Map(bracket.entrants.map((e) => [e.id, e]));
  const standings = bracket.status === "finished" ? bracket.standings : [];

  const rows = standings.map((s) => {
    const team = byId.get(s.team_id);
    const name = team ? teamName(team) : "";
    const solo = team ? soloMember(team) : null;
    // O campeão pode ter perdido uma série (a grande final, vindo de baixo):
    // não foi eliminado.
    const champion = s.rank === 1 && s.tied === 1;
    const out = champion
      ? undefined
      : eliminationOf(bracket.matches, s.team_id);
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
      id: String(s.team_id),
      search: name,
      // A página da pessoa, no time de um jogador que continua na comunidade.
      href: solo && !solo.left ? memberPath(tenant, solo.user_id) : undefined,
      cells: {
        rank:
          s.tied > 1
            ? t("range", { from: s.rank, to: s.rank + s.tied - 1 })
            : t("rank", { rank: s.rank }),
        player: (
          <span className={styles.player}>
            <Avatar src={solo?.avatar_url} name={name} />
            <span className={styles.name}>{name}</span>
          </span>
        ),
        result: (
          <span className={styles.result}>
            {champion && (
              <StatusTag status="confirmed" size="sm">
                {t("champion")}
              </StatusTag>
            )}
            {team && (
              <DepartureTag withdrawn={team.withdrawn} left={teamLeft(team)} />
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
