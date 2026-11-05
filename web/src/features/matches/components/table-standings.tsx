import { useTranslations } from "next-intl";

import { memberPath } from "@/features/members/lib/paths";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import { isEvenSeries } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable, type DataTableColumn } from "@/shared/ui/data-table";
import { InfoTip } from "@/shared/ui/info-tip";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { fragsText } from "../lib/score";
import { DepartureTag } from "./departure-tag";
import styles from "./table-standings.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * A tabela do round robin e do suíço: posição, o time (no duelo, o jogador,
 * com link para a pessoa), o saldo de séries com sinal no round robin ou os
 * pontos no suíço, séries vencidas e perdidas (com os empates da série par e os
 * byes do suíço), partidas com o saldo (na md1 não: seriam as mesmas contas das
 * séries), o saldo de frags (é ele que desempata, não o total; pró e contra
 * vêm ao lado, marcados como incompletos quando alguma série veio sem eles,
 * porque aí não desempatam; "sem frags" quando nenhuma veio), o Buchholz no
 * suíço (explicado no (i) da coluna), e quem venceu, desistiu ou saiu.
 * Embaixo, os desempates que não foram pelo primeiro critério, um por linha ("2º à
 * frente do 3º pelo confronto direto"), visíveis também no celular, onde as
 * colunas de saldo somem (e, com a do Buchholz, o (i): a explicação vem depois
 * dos desempates, se algum foi por ele). Enquanto a fase corre, é a ordem de
 * agora; no fim, a definitiva.
 */
export function TableStandings({ tenant, bracket }: Props) {
  const t = useTranslations("Bracket.table");
  const tStandings = useTranslations("Bracket.standings");
  const teamName = useTeamName();
  const rows = bracket.table ?? [];
  const swiss = bracket.format.format === "swiss";
  const finished = bracket.status === "finished";
  const even = isEvenSeries(bracket.format.best_of);
  const byId = new Map(bracket.entrants.map((e) => [e.id, e]));
  const signed = (n: number) => (n > 0 ? `+${n}` : fragsText(n));
  const nameOf = (teamId: number) => {
    const team = byId.get(teamId);
    return team ? teamName(team) : "";
  };

  const columns: DataTableColumn[] = [
    { key: "position", header: t("position") },
    { key: "team", header: t("team") },
    { key: "score", header: swiss ? t("points") : t("seriesDiff") },
    { key: "record", header: even ? t("recordDraws") : t("record") },
    ...(bracket.format.best_of === 1
      ? []
      : [{ key: "maps", header: t("maps"), secondary: true }]),
    { key: "frags", header: t("frags"), secondary: true },
    ...(swiss
      ? [
          {
            key: "buchholz",
            header: (
              <InfoTip term={t("buchholz")} label={t("buchholzTip")}>
                {t("buchholzMeaning")}
              </InfoTip>
            ),
            secondary: true,
          },
        ]
      : []),
    { key: "notes", header: t("notes"), secondary: true },
  ];

  const tableRows = rows.map((r) => {
    const team = byId.get(r.team_id);
    const name = nameOf(r.team_id);
    const solo = team ? soloMember(team) : null;
    const frags = t("fragsDetail", {
      won: fragsText(r.frags_for),
      lost: fragsText(r.frags_against),
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
        score: swiss ? String(r.points) : signed(r.series_diff),
        record: even
          ? t("recordValueDraws", {
              wins: r.wins,
              draws: r.draws,
              losses: r.losses,
            })
          : r.byes > 0
            ? t("recordByes", { wins: r.wins, losses: r.losses, byes: r.byes })
            : t("recordValue", { wins: r.wins, losses: r.losses }),
        maps: t("diff", {
          won: r.maps_won,
          lost: r.maps_lost,
          diff: signed(r.map_diff),
        }),
        // Sem nenhum frag enviado, "sem frags" em vez de 0–0 incompleto (com
        // negativos, a soma dá 0 com frags; os dois em 0, quase nunca).
        frags:
          !r.frags_complete && r.frags_for === 0 && r.frags_against === 0 ? (
            t("noFrags")
          ) : (
            <span className={styles.frags}>
              <span className={styles.fragDiff}>{signed(r.frag_diff)}</span>
              <span className={styles.fragDetail}>
                {r.frags_complete ? frags : t("incomplete", { frags })}
              </span>
            </span>
          ),
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
  // da seguinte, quando não foi o primeiro critério, que a coluna já mostra.
  // Entre dois que ainda não jogaram, é sempre a ordem da chave: não há o que
  // explicar.
  const started = (r: Schemas["TableRow"]) => r.played + r.byes > 0;
  const tiebreaks = rows.flatMap((r, i) => {
    const next = rows[i + 1];
    if (!next || !r.ahead_by) return [];
    if (r.ahead_by === "points" || r.ahead_by === "series_diff") return [];
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
      title={t("title")}
      description={finished ? undefined : t("current")}
    >
      <DataTable
        label={t("title")}
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
          {rows.some((r) => r.ahead_by === "buchholz") && (
            <p className={styles.buchholzNote}>{t("buchholzMeaning")}</p>
          )}
        </div>
      )}
    </Section>
  );
}
