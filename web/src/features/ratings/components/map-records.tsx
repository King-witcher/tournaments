import { useLocale, useTranslations } from "next-intl";

import { MapTile } from "@/features/maps/components/map-tile";
import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { formatPercent } from "../lib/format-percent";
import styles from "./map-records.module.scss";

type Props = {
  records: components["schemas"]["MapRecord"][];
};

/**
 * "Por mapa" na página da pessoa: um tile por mapa que ela jogou, do mais
 * jogado ao menos, com a imagem (que mostra de relance onde ela vai bem ou
 * mal, útil no veto) e as vitórias e derrotas com a porcentagem. Conta todos
 * os torneios da comunidade, com rating ou não. Sem nenhuma partida, quem
 * chama não mostra a seção.
 */
export function MapRecords({ records }: Props) {
  const t = useTranslations("Ratings");
  const locale = useLocale();
  const detail = ({ wins, losses }: Props["records"][number]) =>
    `${t("record", { wins, losses })} · ${formatPercent(wins, wins + losses, locale)}`;

  return (
    <Section title={t("byMap.title")} description={t("byMap.description")}>
      <ul className={styles.list}>
        {records.map((r) => (
          <MapTile
            key={r.slug}
            slug={r.slug}
            name={r.name}
            detail={detail(r)}
          />
        ))}
      </ul>
    </Section>
  );
}
