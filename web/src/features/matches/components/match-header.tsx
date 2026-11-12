import { useTranslations } from "next-intl";

import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { PageHeader } from "@/shared/ui/page-header";
import { StatusTag } from "@/shared/ui/status-tag";
import { useMatchLabels } from "../hooks/use-match-labels";
import { useSideNames } from "../hooks/use-side-names";
import { taskKey, taskTag } from "../lib/status";
import { MatchStatusLine } from "./match-status-line";
import { VetoSoundToggle } from "./veto-sound-toggle";
import { WalkoverButton } from "./walkover-button";

type Props = {
  tenant: string;
  slug: string;
  /** Nome já no idioma de quem vê. */
  tournamentName: string;
  detail: components["schemas"]["MatchDetail"];
  /** O torneio tem mais de uma fase: a trilha leva à chave da fase da série. */
  severalStages: boolean;
};

/**
 * O topo da série: a trilha até a chave, o código curto (o mesmo do cartão
 * na chave) com os dois lados e, depois da primeira partida enviada, o
 * placar entre eles (o das confirmadas enquanto corre), a etiqueta da
 * contestação em aberto, a parte, a rodada e o melhor de N e, embaixo, como
 * ela terminou, por que ainda não começou ou quem ela espera. Nas
 * ações, o W.O. da organização e, com a série em andamento e veto no torneio, o
 * botão que silencia os sons do veto.
 */
export function MatchHeader({
  tenant,
  slug,
  tournamentName,
  detail,
  severalStages,
}: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
  const s = detail.summary;
  const single = isSingle(detail.format);
  const { code, roundTitle, placeholder } = useMatchLabels(single);
  const names = useSideNames(detail);
  const round = roundTitle(s.part, s.round, detail.part_rounds);
  // A final, a grande final e o reset têm uma partida só: basta o nome.
  const alone = s.part === "grand_final" || s.round === detail.part_rounds;
  const table = s.part === "round_robin" || s.part === "swiss";
  // Na disputa de posições, o título já diz o que está em jogo; a linha diz o
  // formato.
  const label =
    s.part === "placement"
      ? tBracket("parts.placement")
      : table
        ? t("labelTable", {
            round: s.round,
            total: detail.part_rounds,
            index: s.index,
          })
        : alone
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
  const side = (who: "a" | "b") =>
    detail[who] ? names[who] : placeholder(s[who]);
  // O W.O. não tem placar: a linha embaixo diz quem venceu.
  const r = s.result;
  const score = r
    ? r.walkover || r.a_wins == null || r.b_wins == null
      ? null
      : { a: r.a_wins, b: r.b_wins }
    : detail.games.length > 0
      ? { a: s.a_maps, b: s.b_maps }
      : null;
  const sides = { code: code(s.key), a: side("a"), b: side("b") };
  const contested = s.task && taskTag(s.task) === "contested" ? s.task : null;
  // O W.O. vale na série pronta e, no lugar do resultado, na decidida.
  const canWalkover =
    detail.can_manage && (s.state === "ready" || s.state === "completed");
  const sound = s.state === "ready" && detail.veto != null;

  return (
    <PageHeader
      breadcrumbs={[
        { href: `/${tenant}/${slug}`, label: tournamentName },
        {
          href: severalStages
            ? {
                pathname: `/${tenant}/${slug}/bracket`,
                query: { stage: detail.stage },
              }
            : `/${tenant}/${slug}/bracket`,
          label: tBracket("title"),
        },
      ]}
      title={
        score
          ? t("titleScore", { ...sides, aScore: score.a, bScore: score.b })
          : t("title", sides)
      }
      // Só a contestação tem etiqueta: o estado está nos passos e na linha
      // embaixo, a ação de quem vê é o passo em que a página abre e a espera
      // é texto (`MatchStatusLine`).
      tags={
        contested && (
          <StatusTag status="contested">
            {tBracket(`task.${taskKey(contested, s.veto)}`)}
          </StatusTag>
        )
      }
      meta={`${label} · ${t("bestOf", { count: s.best_of })}`}
      actions={
        (canWalkover || sound) && (
          <>
            {canWalkover && (
              <WalkoverButton
                target={{ tenant, slug, matchKey: s.key }}
                names={names}
                decided={s.state === "completed"}
              />
            )}
            {sound && <VetoSoundToggle />}
          </>
        )
      }
    >
      <MatchStatusLine detail={detail} />
    </PageHeader>
  );
}
