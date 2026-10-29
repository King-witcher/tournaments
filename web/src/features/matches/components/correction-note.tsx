import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { Fragment } from "react";

import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
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
 * jogou uma partida seguinte que também está decidida: o aviso, abaixo dos
 * mapas, de que a correção que muda o resultado desta é recusada, por quê e
 * o que fazer antes de tentar, com o link da partida seguinte. Some quando
 * não há nenhuma.
 */
export function CorrectionNote({ bracket, matchKey, basePath }: Props) {
  const t = useTranslations("Match.correction");
  const { code } = useMatchLabels(isSingle(bracket.format));
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
      <Link href={`${basePath}/${m.key}`} inline>
        {code(m.key)}
      </Link>
    </Fragment>
  ));
  return (
    <Notice kind="warning" title={t("title")}>
      <Stack gap={3}>
        <p>{t.rich("why", { matches: () => links })}</p>
        <p>
          {t.rich(walkover ? "howWalkover" : "how", { matches: () => links })}
        </p>
        <p>{t("allowed")}</p>
      </Stack>
    </Notice>
  );
}
