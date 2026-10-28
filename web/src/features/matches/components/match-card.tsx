import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Link } from "@/shared/ui/link";
import { useMatchLabels } from "../hooks/use-match-labels";
import { ACTIONABLE } from "../lib/status";
import styles from "./match-card.module.scss";
import { MatchCardSide } from "./match-card-side";
import { MatchCardStatus } from "./match-card-status";
import { MemberLink } from "./member-link";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  match: Schemas["BracketMatch"];
  single: boolean;
  entrants: Map<number, Schemas["BracketEntrant"]>;
  href: string;
};

/**
 * Os placares de cada lado no cartão: o da série decidida ou, enquanto ela
 * corre, os mapas confirmados de cada um.
 */
function scores(
  m: Schemas["BracketMatch"],
  walkover: string,
): [string | null, string | null] {
  const r = m.result;
  if (r?.walkover) return r.winner === "a" ? [walkover, "–"] : ["–", walkover];
  if (r) return [String(r.a_wins), String(r.b_wins)];
  const played = m.a_maps + m.b_maps + m.pending_maps + m.contested_maps > 0;
  if (m.state === "ready" && played)
    return [String(m.a_maps), String(m.b_maps)];
  return [null, null];
}

/**
 * Uma partida na chave: o código (o link da partida, que cobre o cartão
 * inteiro), os dois lados (seed, apelido como link para a pessoa, placar) e
 * o que ela espera de quem vê. As de quem vê têm a borda na cor de
 * interação; as que pedem ação, também a faixa à esquerda. Bye e reset que
 * não precisou ficam discretos, sem link.
 */
export function MatchCard({ tenant, match: m, single, entrants, href }: Props) {
  const t = useTranslations("Bracket");
  const { code, placeholder } = useMatchLabels(single);

  if (m.state === "bye" || m.state === "not_needed") {
    const who = entrants.get(m.a.user_id ?? m.b.user_id ?? -1);
    return (
      <p className={styles.ghost}>
        {m.state === "not_needed"
          ? `${code(m.key)} · ${t("notNeeded")}`
          : who
            ? t.rich("advances", {
                name: () => (
                  <MemberLink tenant={tenant} userId={who.user_id} inline>
                    {who.nickname}
                  </MemberLink>
                ),
              })
            : t("bye")}
      </p>
    );
  }

  const [scoreA, scoreB] = scores(m, t("walkover"));
  const confirmed = m.state === "completed";
  const winner = confirmed ? m.result?.winner : undefined;
  const actionable = m.task != null && ACTIONABLE.includes(m.task);
  const name = (side: Schemas["MatchSide"]) =>
    (side.user_id != null && entrants.get(side.user_id)?.nickname) ||
    placeholder(side);

  return (
    <div
      className={cn(
        styles.card,
        m.yours && styles.yours,
        actionable && styles.actionable,
      )}
    >
      <div className={styles.head}>
        {/* O código sublinhado na cor de link mostra que o cartão abre a
            partida; quem usa leitor de tela ouve também os dois lados. */}
        <Link
          href={href}
          inline
          size="sm"
          className={styles.open}
          aria-label={`${code(m.key)} ${t("matchSides", { a: name(m.a), b: name(m.b) })}`}
        >
          {code(m.key)}
        </Link>
        <span className={styles.bestOf}>
          {t("bestOf", { count: m.best_of })}
        </span>
      </div>
      <MatchCardSide
        tenant={tenant}
        side={m.a}
        entrants={entrants}
        single={single}
        score={scoreA}
        winner={winner === "a"}
        tentative={!confirmed}
      />
      <MatchCardSide
        tenant={tenant}
        side={m.b}
        entrants={entrants}
        single={single}
        score={scoreB}
        winner={winner === "b"}
        tentative={!confirmed}
      />
      <MatchCardStatus match={m} />
    </div>
  );
}
