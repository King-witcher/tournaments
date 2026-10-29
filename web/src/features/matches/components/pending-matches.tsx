import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { Section } from "@/shared/ui/section";
import { useMatchLabels } from "../hooks/use-match-labels";
import { ACTIONABLE, taskKey } from "../lib/status";
import styles from "./pending-matches.module.scss";

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
 * organização, resolver), cada uma como link com os lados e o que falta: numa
 * chave grande ou no celular, o cartão da partida pode estar fora da tela.
 * Some sem pendência.
 */
export function PendingMatches({ bracket, basePath, exclude }: Props) {
  const t = useTranslations("Bracket");
  const { code } = useMatchLabels(isSingle(bracket.format));
  const teamName = useTeamName();
  const teams = new Map(bracket.entrants.map((e) => [e.id, e]));
  const pending = bracket.matches.filter(
    (m) => m.key !== exclude && m.task && ACTIONABLE.includes(m.task),
  );
  if (pending.length === 0) return null;

  const name = (side: Schemas["MatchSide"]) => {
    const team = side.team_id != null ? teams.get(side.team_id) : undefined;
    return team ? teamName(team) : t("tbd");
  };
  return (
    <Section title={t("pending.title")}>
      <ul className={styles.list}>
        {pending.map((m) => (
          <li key={m.key}>
            <Link href={`${basePath}/${m.key}`}>
              {t("pending.item", {
                match: code(m.key),
                a: name(m.a),
                b: name(m.b),
                task: m.task ? t(`task.${taskKey(m.task, m.veto)}`) : "",
              })}
            </Link>
          </li>
        ))}
      </ul>
    </Section>
  );
}
