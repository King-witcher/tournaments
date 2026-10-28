import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";

type Props = {
  tenant: string;
  slug: string;
  /** Nome já no idioma de quem vê. */
  tournamentName: string;
  detail: components["schemas"]["MatchDetail"];
};

/**
 * Torneio (link de volta), parte, rodada e número da partida, o código curto
 * (o mesmo do cartão na chave) e melhor de N.
 */
export function MatchHeader({ tenant, slug, tournamentName, detail }: Props) {
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
      <Link
        href={`/${tenant}/${slug}`}
        className="text-sm opacity-70 hover:underline"
      >
        {tournamentName}
      </Link>
      <h1 className="text-2xl font-semibold">{label}</h1>
      <p className="text-sm opacity-70">
        {code(s.key)} · {t("bestOf", { count: s.best_of })}
      </p>
    </header>
  );
}
