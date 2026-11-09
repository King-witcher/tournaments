import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { memberPath } from "@/features/members/lib/paths";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { Stack } from "@/shared/ui/carbon/stack";
import { DataTable, type DataTableColumn } from "@/shared/ui/data-table";
import { InfoTip } from "@/shared/ui/info-tip";
import { StatusTag } from "@/shared/ui/status-tag";
import { STANDINGS_TABLE_ID } from "../lib/ties";
import { DepartureTag } from "./departure-tag";
import {
  type NumberColumn,
  numberColumns,
  useRatingText,
  useStandingNumbers,
} from "./standing-numbers";
import styles from "./table-standings.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  bracket: Schemas["TournamentBracket"];
  /** É a última fase: o 1º, com ela encerrada, venceu o torneio. */
  lastStage: boolean;
};

/**
 * A tabela do round robin e do suíço: posição, o time (no duelo, o jogador,
 * com link para a pessoa), os pontos no suíço, e as colunas de número na ordem
 * dos desempates da fase (`numberColumns`), o saldo em cima e, embaixo, de
 * onde ele vem (`useStandingNumbers`); no fim, quem venceu, desistiu ou saiu.
 * Na linha que ficou à frente da seguinte por um critério, que não o primeiro
 * (a coluna já o mostra), um (i) ao lado da posição com a frase ("2º à frente
 * do 3º pelo confronto direto"; no menor rating, com o rating do início de
 * cada um; na decisão da organização, com o motivo); ele fica fora das
 * colunas que somem no celular. Enquanto a fase corre, é a ordem de agora; no
 * fim, a definitiva. No empate do fim que espera a organização, os empatados
 * dividem a melhor posição do grupo, com a etiqueta "Empate a decidir".
 */
export function TableStandings({ tenant, bracket, lastStage }: Props) {
  const t = useTranslations("Bracket.table");
  const tStandings = useTranslations("Bracket.standings");
  const teamName = useTeamName();
  const numbers = useStandingNumbers(bracket.format.best_of);
  const ratingOf = useRatingText();
  const rows = bracket.table ?? [];
  const swiss = bracket.format.format === "swiss";
  const finished = bracket.status === "finished";
  const byId = new Map(bracket.entrants.map((e) => [e.id, e]));
  const nameOf = (teamId: number) => {
    const team = byId.get(teamId);
    return team ? teamName(team) : "";
  };

  const headers: Record<NumberColumn, ReactNode> = {
    record: t("record"),
    maps: t("maps"),
    frags: t("frags"),
    buchholz: (
      <InfoTip term={t("buchholz")} label={t("buchholzTip")}>
        {t("buchholzMeaning")}
      </InfoTip>
    ),
  };
  const columns: DataTableColumn[] = [
    { key: "position", header: t("position") },
    { key: "team", header: t("team") },
    ...(swiss ? [{ key: "points", header: t("points") }] : []),
    ...numberColumns(bracket.format).map((key) => ({
      key,
      header: headers[key],
      secondary: key !== "record",
    })),
    { key: "notes", header: t("notes"), secondary: true },
  ];

  // O critério que pôs a linha à frente da seguinte, quando não foi o
  // primeiro. Entre dois que ainda não jogaram, é sempre a ordem da chave: não
  // há o que explicar. No empate a decidir, a ordem é provisória.
  const started = (r: Schemas["TableRow"]) => r.played + r.byes > 0;
  const whyAhead = (r: Schemas["TableRow"], i: number) => {
    const next = rows[i + 1];
    if (!next || !r.ahead_by) return undefined;
    if (r.ahead_by === "points" || r.ahead_by === "series_diff")
      return undefined;
    if (!started(r) && !started(next)) return undefined;
    if (r.ahead_by === "seed" && r.tie_pending && next.tie_pending)
      return undefined;
    const values = {
      position: r.position,
      name: nameOf(r.team_id),
      next: next.position,
      other: nameOf(next.team_id),
      rating: ratingOf(r),
      otherRating: ratingOf(next),
    };
    if (r.ahead_by === "organizer" && r.tie_reason) {
      return t("aheadBy.organizerReason", { ...values, reason: r.tie_reason });
    }
    const text = t(`aheadBy.${r.ahead_by}`, values);
    // No celular, a coluna do Buchholz (e o (i) dela) some.
    return r.ahead_by === "buchholz" ? `${text} ${t("buchholzMeaning")}` : text;
  };

  const tableRows = rows.map((r, i) => {
    const team = byId.get(r.team_id);
    const name = nameOf(r.team_id);
    const solo = team ? soloMember(team) : null;
    const place = t("place", { position: r.position });
    const why = whyAhead(r, i);
    return {
      id: String(r.team_id),
      search: name,
      // A página da pessoa, no time de um jogador que continua na comunidade.
      href: solo && !solo.left ? memberPath(tenant, solo.user_id) : undefined,
      cells: {
        position: why ? (
          <InfoTip term={place} label={t("whyAhead", { name })}>
            {why}
          </InfoTip>
        ) : (
          place
        ),
        team: (
          <span className={styles.team}>
            <Avatar src={solo?.avatar_url} name={name} />
            <span className={styles.name}>{name}</span>
          </span>
        ),
        points: String(r.points),
        ...numbers(r),
        notes: (
          <span className={styles.notes}>
            {finished && lastStage && r.position === 1 && (
              <StatusTag status="confirmed" size="sm">
                {tStandings("champion")}
              </StatusTag>
            )}
            {r.tie_pending && (
              <StatusTag status="in_progress" size="sm">
                {t("tiePending")}
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

  return (
    // Sem título: o da página já é "Classificação".
    <Stack gap={5} id={STANDINGS_TABLE_ID} tabIndex={-1}>
      {!finished && (bracket.pending_ties ?? []).length === 0 && (
        <p className={styles.current}>{t("current")}</p>
      )}
      <DataTable
        label={t("title")}
        primaryColumn="team"
        columns={columns}
        rows={tableRows}
        empty={{ text: t("empty") }}
      />
    </Stack>
  );
}
