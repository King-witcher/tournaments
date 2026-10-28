import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { PageHeader } from "@/shared/ui/page-header";
import { StatusTag } from "@/shared/ui/status-tag";
import { useMatchLabels } from "../hooks/use-match-labels";
import { MATCH_STATE_STATUS, TASK_STATUS } from "../lib/status";

type Props = {
  tenant: string;
  slug: string;
  tenantName: string;
  /** Nome já no idioma de quem vê. */
  tournamentName: string;
  detail: components["schemas"]["MatchDetail"];
};

/**
 * O topo da partida: a trilha até a chave, o código curto (o mesmo do
 * cartão na chave) com os dois lados, o estado e o que ela espera de quem
 * vê, e a parte, a rodada e o melhor de N.
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
  const tNav = useTranslations("Nav");
  const s = detail.summary;
  const single = detail.format.bracket === "single";
  const { code, roundTitle, placeholder } = useMatchLabels(single);
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
  const side = (who: "a" | "b") => detail[who]?.nickname ?? placeholder(s[who]);

  return (
    <PageHeader
      breadcrumbs={[
        { href: "/", label: tNav("home") },
        { href: `/${tenant}`, label: tenantName },
        { href: `/${tenant}/${slug}`, label: tournamentName },
        { href: `/${tenant}/${slug}/bracket`, label: tBracket("title") },
      ]}
      title={t("title", { code: code(s.key), a: side("a"), b: side("b") })}
      tags={
        <>
          <StatusTag status={MATCH_STATE_STATUS[s.state]}>
            {t(`stateTag.${s.state}`)}
          </StatusTag>
          {s.task && (
            <StatusTag status={TASK_STATUS[s.task]}>
              {tBracket(`task.${s.task}`)}
            </StatusTag>
          )}
        </>
      }
      meta={`${label} · ${t("bestOf", { count: s.best_of })}`}
    />
  );
}
