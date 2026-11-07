import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { Section } from "@/shared/ui/section";
import { useMatchLabels } from "../hooks/use-match-labels";
import { actionableMatches, matchProgress } from "../lib/status";
import styles from "./pending-matches.module.scss";

type Schemas = components["schemas"];

type Props = {
  bracket: Schemas["TournamentBracket"];
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
};

/**
 * As séries que estão sendo jogadas agora (veto começado ou alguma partida
 * enviada), com os lados e o placar, cada uma como link: o que quem acompanha
 * procura sem varrer a chave. Fora as que já estão em "Séries que esperam
 * você". Some sem nenhuma.
 */
export function LiveMatches({ bracket, basePath }: Props) {
  const t = useTranslations("Bracket");
  const { code } = useMatchLabels(isSingle(bracket.format));
  const teamName = useTeamName();
  const teams = new Map(bracket.entrants.map((e) => [e.id, e]));
  const mine = new Set(actionableMatches(bracket).map((m) => m.key));
  const live = bracket.matches.filter(
    (m) =>
      m.state !== "completed" &&
      matchProgress(m) === "in_progress" &&
      !mine.has(m.key),
  );
  if (live.length === 0) return null;

  const name = (side: Schemas["MatchSide"]) => {
    const team = side.team_id != null ? teams.get(side.team_id) : undefined;
    return team ? teamName(team) : t("tbd");
  };
  return (
    <Section title={t("live.title")}>
      <ul className={styles.list}>
        {live.map((m) => (
          <li key={m.key}>
            <Link href={`${basePath}/${m.key}`}>
              {t("live.item", {
                match: code(m.key),
                a: name(m.a),
                b: name(m.b),
                aMaps: m.a_maps,
                bMaps: m.b_maps,
              })}
            </Link>
          </li>
        ))}
      </ul>
    </Section>
  );
}
