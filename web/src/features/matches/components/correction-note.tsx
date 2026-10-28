import { useTranslations } from "next-intl";
import { Fragment } from "react";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";

type Schemas = components["schemas"];

type Props = {
  bracket: Schemas["TournamentBracket"];
  /** A partida da página, decidida. */
  matchKey: string;
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
};

/**
 * Para a organização, numa série decidida cujo vencedor (ou perdedor) já
 * jogou uma partida seguinte que também está decidida: a correção que muda o
 * resultado desta é recusada, e a nota diz por quê e o que fazer antes de
 * tentar, com o link da partida seguinte. Some quando não há nenhuma.
 */
export function CorrectionNote({ bracket, matchKey, basePath }: Props) {
  const t = useTranslations("Match.correction");
  const { code } = useMatchLabels(bracket.format.bracket === "single");
  const later = bracket.matches.filter(
    (m) =>
      m.state === "completed" &&
      (m.a.from?.match_key === matchKey || m.b.from?.match_key === matchKey),
  );
  if (later.length === 0) return null;
  // W.O. não volta a ficar sem decisão por correção de mapa.
  const walkover = later.some((m) => m.result?.walkover);

  const links = later.map((m, i) => (
    <Fragment key={m.key}>
      {i > 0 && ", "}
      <Link
        href={`${basePath}/${m.key}`}
        className="text-link underline underline-offset-4"
      >
        {code(m.key)}
      </Link>
    </Fragment>
  ));
  return (
    <section className="flex flex-col gap-1 rounded-md border border-amber-500/60 p-3 text-sm">
      <h2 className="font-semibold">{t("title")}</h2>
      <p>{t.rich("why", { matches: () => links })}</p>
      <p>
        {t.rich(walkover ? "howWalkover" : "how", { matches: () => links })}
      </p>
      <p className="text-muted">{t("allowed")}</p>
    </section>
  );
}
