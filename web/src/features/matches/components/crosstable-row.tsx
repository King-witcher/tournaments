import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { TableCell, TableRow } from "@/shared/ui/carbon/data-table";
import type { CrossTeam } from "../lib/crosstable";
import { CrosstableCell } from "./crosstable-cell";
import styles from "./crosstable-row.module.scss";

type Props = {
  tenant: string;
  /** Slug do torneio. */
  tournament: string;
  row: CrossTeam;
  /** Todos os times, na ordem das colunas. */
  teams: CrossTeam[];
  /** A série entre dois times. */
  series: (
    x: number,
    y: number,
  ) => components["schemas"]["BracketMatch"] | undefined;
};

/** Uma linha da tabela cruzada: o time e o resultado dele contra cada coluna. */
export function CrosstableRow({
  tenant,
  tournament,
  row,
  teams,
  series,
}: Props) {
  return (
    <TableRow className={styles.row}>
      <th scope="row" className={styles.rowHeader} title={row.name}>
        <span className={styles.label}>
          <span className={styles.position}>{row.position}</span>
          <span className={styles.avatar}>
            <Avatar src={row.avatar} name={row.name} />
          </span>
          <span className={styles.name}>{row.name}</span>
        </span>
      </th>
      {teams.map((column) => {
        const m = column.id === row.id ? null : series(row.id, column.id);
        return (
          <TableCell
            key={column.id}
            className={m ? styles.cell : styles.diagonal}
          >
            {m && (
              <CrosstableCell
                match={m}
                team={row.id}
                row={row.name}
                column={column.name}
                href={`/${tenant}/${tournament}/matches/${m.key}`}
              />
            )}
          </TableCell>
        );
      })}
    </TableRow>
  );
}
