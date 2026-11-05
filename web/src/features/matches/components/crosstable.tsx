import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from "@/shared/ui/carbon/data-table";
import { seriesByPair } from "../lib/crosstable";
import styles from "./crosstable.module.scss";
import { CrosstableRow } from "./crosstable-row";

type Props = {
  tenant: string;
  /** Slug do torneio. */
  tournament: string;
  bracket: components["schemas"]["TournamentBracket"];
};

/**
 * Quem enfrentou quem, no round robin: uma linha e uma coluna por time, na
 * ordem da classificação, e cada célula é o resultado de quem está na linha.
 * A coluna diz a posição e o nome cortado (inteiro na dica e para o leitor de
 * tela). Larga demais, rola dentro da caixa da tabela (a do `Table` do Carbon),
 * com a primeira coluna fixa; a página não rola para o lado.
 */
export function Crosstable({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket.crosstable");
  const teamName = useTeamName();
  const entrants = new Map(bracket.entrants.map((e) => [e.id, e]));
  const series = seriesByPair(bracket.matches);
  const teams = (bracket.table ?? []).flatMap((row) => {
    const team = entrants.get(row.team_id);
    if (!team) return [];
    return [
      {
        id: team.id,
        position: row.position,
        name: teamName(team),
        avatar: soloMember(team)?.avatar_url,
      },
    ];
  });
  const hasFrags = bracket.matches.some((m) => m.best_of === 1);

  return (
    <div className={styles.crosstable}>
      <p className={styles.legend}>
        {t("legend")}
        {hasFrags && ` ${t("legendFrags")}`}
      </p>
      <Table
        aria-label={t("label")}
        size="md"
        useStaticWidth
        className={styles.table}
      >
        <TableHead>
          <TableRow>
            <TableHeader className={styles.corner}>
              <span className={styles.cornerLabel}>{t("corner")}</span>
            </TableHeader>
            {teams.map((team) => (
              <TableHeader
                key={team.id}
                className={styles.columnHeader}
                title={team.name}
              >
                <span className={styles.columnPosition}>{team.position}</span>
                <span className={styles.columnName}>{team.name}</span>
              </TableHeader>
            ))}
          </TableRow>
        </TableHead>
        <TableBody>
          {teams.map((row) => (
            <CrosstableRow
              key={row.id}
              tenant={tenant}
              tournament={tournament}
              row={row}
              teams={teams}
              series={series}
            />
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
