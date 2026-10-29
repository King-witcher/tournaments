import { useTranslations } from "next-intl";

import { memberPath } from "@/features/members/lib/paths";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable, type DataTableColumn } from "@/shared/ui/data-table";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { DepartureTag } from "./departure-tag";
import styles from "./table-standings.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * A tabela do round robin e do suíço: posição, o time (no duelo, o jogador,
 * com link para a pessoa), pontos, vitórias e derrotas (com os byes do
 * suíço), mapas e frags com o saldo (os frags marcados como incompletos
 * quando alguma série veio sem eles, porque aí não desempatam; "sem frags"
 * quando nenhuma veio), o Buchholz no suíço (explicado uma vez, na
 * descrição), e quem venceu, desistiu ou saiu. Embaixo, os desempates que não
 * foram pelos pontos, um por linha ("2º à frente do 3º pelo confronto
 * direto"), visíveis também no celular, onde as colunas de saldo somem.
 * Enquanto a fase corre, é a ordem de agora; no fim, a classificação.
 */
export function TableStandings({ tenant, bracket }: Props) {
  const t = useTranslations("Bracket.table");
  const tStandings = useTranslations("Bracket.standings");
  const teamName = useTeamName();
  const rows = bracket.table ?? [];
  const swiss = bracket.format.format === "swiss";
  const finished = bracket.status === "finished";
  const byId = new Map(bracket.entrants.map((e) => [e.id, e]));
  const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
  const nameOf = (teamId: number) => {
    const team = byId.get(teamId);
    return team ? teamName(team) : "";
  };

  const columns: DataTableColumn[] = [
    { key: "position", header: t("position") },
    { key: "team", header: t("team") },
    { key: "points", header: t("points") },
    { key: "record", header: t("record") },
    { key: "maps", header: t("maps"), secondary: true },
    { key: "frags", header: t("frags"), secondary: true },
    ...(swiss
      ? [{ key: "buchholz", header: t("buchholz"), secondary: true }]
      : []),
    { key: "notes", header: t("notes"), secondary: true },
  ];

  const tableRows = rows.map((r) => {
    const team = byId.get(r.team_id);
    const name = nameOf(r.team_id);
    const solo = team ? soloMember(team) : null;
    const frags = t("diff", {
      won: r.frags_for,
      lost: r.frags_against,
      diff: signed(r.frag_diff),
    });
    return {
      id: String(r.team_id),
      search: name,
      // A página da pessoa, no time de um jogador que continua na comunidade.
      href: solo && !solo.left ? memberPath(tenant, solo.user_id) : undefined,
      cells: {
        position: t("place", { position: r.position }),
        team: (
          <span className={styles.team}>
            <Avatar src={solo?.avatar_url} name={name} />
            <span className={styles.name}>{name}</span>
          </span>
        ),
        points: String(r.points),
        record:
          r.byes > 0
            ? t("recordByes", { wins: r.wins, losses: r.losses, byes: r.byes })
            : t("recordValue", { wins: r.wins, losses: r.losses }),
        maps: t("diff", {
          won: r.maps_won,
          lost: r.maps_lost,
          diff: signed(r.map_diff),
        }),
        // Sem nenhum frag enviado, "sem frags" em vez de 0–0 incompleto.
        frags: r.frags_complete
          ? frags
          : r.frags_for + r.frags_against === 0
            ? t("noFrags")
            : t("incomplete", { frags }),
        buchholz: String(r.buchholz ?? 0),
        notes: (
          <span className={styles.notes}>
            {finished && r.position === 1 && (
              <StatusTag status="confirmed" size="sm">
                {tStandings("champion")}
              </StatusTag>
            )}
            {team && (
              <DepartureTag withdrawn={team.withdrawn} left={teamLeft(team)} />
            )}
          </span>
        ),
      },
    };
  });

  // Os desempates que a tela explica: o critério que pôs cada linha à frente
  // da seguinte, quando não foram os pontos. Entre dois que ainda não
  // jogaram, é sempre a ordem da chave: não há o que explicar.
  const started = (r: Schemas["TableRow"]) => r.played + r.byes > 0;
  const tiebreaks = rows.flatMap((r, i) => {
    const next = rows[i + 1];
    if (!next || !r.ahead_by || r.ahead_by === "points") return [];
    if (!started(r) && !started(next)) return [];
    return [
      t(`aheadBy.${r.ahead_by}`, {
        position: r.position,
        name: nameOf(r.team_id),
        next: next.position,
        other: nameOf(next.team_id),
      }),
    ];
  });

  return (
    <Section
      title={finished ? t("finalTitle") : t("title")}
      description={
        [!finished && t("current"), swiss && t("buchholzMeaning")]
          .filter(Boolean)
          .join(" ") || undefined
      }
    >
      <DataTable
        label={finished ? t("finalTitle") : t("title")}
        primaryColumn="team"
        columns={columns}
        rows={tableRows}
        empty={{ text: t("empty") }}
      />
      {tiebreaks.length > 0 && (
        <div className={styles.tiebreaks}>
          <h3 className={styles.tiebreaksTitle}>{t("tiebreaks")}</h3>
          <ul className={styles.tiebreakList}>
            {tiebreaks.map((text) => (
              <li key={text}>{text}</li>
            ))}
          </ul>
        </div>
      )}
    </Section>
  );
}
