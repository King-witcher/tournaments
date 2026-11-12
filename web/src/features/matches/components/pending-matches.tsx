import { useTranslations } from "next-intl";

import { severity, warningsFirst } from "@/features/organizing/lib/severity";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import { TaskNoticeList } from "@/shared/ui/task-notice-list";
import { useMatchLabels } from "../hooks/use-match-labels";
import { useTaskNoticeText } from "../hooks/use-task-notice-text";
import { actionableMatches } from "../lib/status";

type Schemas = components["schemas"];

type Props = {
  bracket: Schemas["TournamentBracket"];
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
  /** Partida que não entra na lista (a da página em que ela aparece). */
  exclude?: string;
};

/**
 * As séries que esperam uma ação de quem vê (enviar, responder, o veto ou,
 * para a organização, resolver), um aviso por série: a situação no título
 * (`useTaskNoticeText`, o mesmo da home), a série com os lados embaixo e, se
 * for o caso, o próximo mapa, o prazo da resposta ou o aceite da
 * contestação: numa chave grande ou no celular, o cartão da série pode estar
 * fora da tela. O botão leva à série. As contestações primeiro
 * (`warningsFirst`). Some sem pendência.
 */
export function PendingMatches({ bracket, basePath, exclude }: Props) {
  const t = useTranslations("Bracket");
  const { code } = useMatchLabels(isSingle(bracket.format));
  const { title, note } = useTaskNoticeText();
  const teamName = useTeamName();
  const teams = new Map(bracket.entrants.map((e) => [e.id, e]));
  const pending = warningsFirst(
    actionableMatches(bracket, exclude).map((match) => ({
      match,
      kind: severity(match.task),
    })),
  );
  if (pending.length === 0) return null;

  const name = (side: Schemas["MatchSide"]) => {
    const team = side.team_id != null ? teams.get(side.team_id) : undefined;
    return team ? teamName(team) : t("tbd");
  };
  return (
    <TaskNoticeList label={t("pending.title")}>
      {pending.map(({ match: m, kind }) => {
        const detail = m.task_detail;
        const side = detail?.viewer_side;
        const opponent = side ? name(side === "a" ? m.b : m.a) : undefined;
        return (
          <TaskNotice
            key={m.key}
            kind={kind}
            title={title(m.task, detail, opponent)}
            meta={
              <MetaLines
                lines={[
                  t("pending.item", {
                    match: code(m.key),
                    a: name(m.a),
                    b: name(m.b),
                  }),
                  m.task === "report" && m.next_map
                    ? t("nextMap", { map: m.next_map.name })
                    : note(m.task, detail),
                ]}
              />
            }
            action={
              <ButtonLink href={`${basePath}/${m.key}`} kind="ghost" size="sm">
                {t("pending.open")}
              </ButtonLink>
            }
          />
        );
      })}
    </TaskNoticeList>
  );
}
