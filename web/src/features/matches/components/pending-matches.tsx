import { useTranslations } from "next-intl";

import { severity, warningsFirst } from "@/features/organizing/lib/severity";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { TaskNotice } from "@/shared/ui/task-notice";
import { TaskNoticeList } from "@/shared/ui/task-notice-list";
import { useMatchLabels } from "../hooks/use-match-labels";
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
 * As partidas que esperam uma ação de quem vê (enviar, responder ou, para a
 * organização, resolver), um aviso por partida com os lados, o botão do que
 * falta e, nas que ele joga, o próximo mapa quando já saiu: numa chave grande
 * ou no celular, o cartão da partida pode estar fora da tela. As contestações
 * primeiro (`warningsFirst`). Some sem pendência.
 */
export function PendingMatches({ bracket, basePath, exclude }: Props) {
  const t = useTranslations("Bracket");
  const { code } = useMatchLabels(isSingle(bracket.format));
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
      {pending.map(({ match: m, kind }) => (
        <TaskNotice
          key={m.key}
          kind={kind}
          title={t("pending.item", {
            match: code(m.key),
            a: name(m.a),
            b: name(m.b),
          })}
          meta={m.yours && m.next_map && t("nextMap", { map: m.next_map.name })}
          action={
            <ButtonLink href={`${basePath}/${m.key}`} kind="ghost" size="sm">
              {t(`pending.action.${m.task}`)}
            </ButtonLink>
          }
        />
      ))}
    </TaskNoticeList>
  );
}
