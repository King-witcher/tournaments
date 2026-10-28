import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";

type Props = {
  tenant: string;
  slug: string;
  tenantName: string;
  /** Nome já no idioma de quem vê. */
  tournamentName: string;
  detail: components["schemas"]["MatchDetail"];
};

/**
 * Comunidade e torneio (links de volta), parte, rodada e número da partida,
 * o código curto (o mesmo do cartão na chave) e melhor de N.
 */
export function MatchHeader({
  tenant,
  slug,
  tenantName,
  tournamentName,
  detail,
}: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
  const s = detail.summary;
  const single = detail.format.bracket === "single";
  const { code, roundTitle } = useMatchLabels(single);
  const round = roundTitle(s.part, s.round, detail.part_rounds);
  // A final, a grande final e o reset têm uma partida só: basta o nome.
  const alone = s.part === "grand_final" || s.round === detail.part_rounds;
  const label = alone
    ? single || s.part === "grand_final"
      ? round
      : t("labelPart", { part: tBracket(`parts.${s.part}`), round })
    : single
      ? t("labelSingle", { round, index: s.index })
      : t("label", {
          part: tBracket(`parts.${s.part}`),
          round,
          index: s.index,
        });

  return (
    <header className="flex flex-col gap-1">
      <nav className="flex flex-wrap gap-x-2 text-sm text-muted">
        <Link
          href={`/${tenant}`}
          className="underline underline-offset-4 hover:decoration-2"
        >
          {tenantName}
        </Link>
        <span aria-hidden>›</span>
        <Link
          href={`/${tenant}/${slug}`}
          className="underline underline-offset-4 hover:decoration-2"
        >
          {tournamentName}
        </Link>
      </nav>
      <h1 className="text-2xl font-semibold">{label}</h1>
      <p className="text-sm text-muted">
        {code(s.key)} · {t("bestOf", { count: s.best_of })}
      </p>
    </header>
  );
}
