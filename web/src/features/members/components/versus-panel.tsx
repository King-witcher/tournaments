import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";

type Props = {
  /** O apelido da pessoa da página. */
  nickname: string;
  /** A linha de quem vê na lista da pessoa (do ponto de vista dela); `undefined` sem confronto. */
  record?: components["schemas"]["OpponentRecord"];
};

/**
 * "Você contra {apelido}", no topo da aba Confrontos do perfil de outra
 * pessoa: séries e partidas do ponto de vista de quem vê (a linha dele na
 * lista dela, invertida) e o último confronto, na modalidade escolhida.
 */
export function VersusPanel({ nickname, record }: Props) {
  const t = useTranslations("Member.opponents");
  const tCareer = useTranslations("Member.career");
  const title = t("versus", { name: nickname });
  if (!record) {
    return (
      <Panel>
        <Section title={title}>
          <p>{t("versusNone")}</p>
        </Section>
      </Panel>
    );
  }
  // A linha é da pessoa contra quem vê: as vitórias dela são as derrotas dele.
  const { series, games } = record;
  const seriesText =
    series.draws > 0
      ? tCareer("recordDraws", {
          wins: series.losses,
          draws: series.draws,
          losses: series.wins,
        })
      : tCareer("record", { wins: series.losses, losses: series.wins });
  return (
    <Panel>
      <Section title={title}>
        <StructuredList
          label={title}
          condensed
          rows={[
            { key: "series", cells: [t("series"), seriesText] },
            {
              key: "games",
              cells: [
                t("games"),
                tCareer("record", { wins: games.losses, losses: games.wins }),
              ],
            },
            {
              key: "last",
              cells: [
                t("last"),
                <LocalDateTime key="last" value={record.last_at} />,
              ],
            },
          ]}
        />
      </Section>
    </Panel>
  );
}
