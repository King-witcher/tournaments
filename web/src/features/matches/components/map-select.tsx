"use client";

import { useTranslations } from "next-intl";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import type { components } from "@/shared/api/schema";
import { Dropdown } from "@/shared/ui/dropdown";
import { RequiredLabel } from "@/shared/ui/required-label";
import styles from "./map-select.module.scss";

type Map = components["schemas"]["Map"];

type Props = {
  maps: Map[];
  value: string;
  onChange: (slug: string) => void;
};

/**
 * O mapa jogado, numa lista com a miniatura de cada mapa ao lado do nome
 * (também no escolhido): o pool de duelo primeiro, depois os outros. Quem
 * chama já tirou os mapas que não cabem (os já jogados na série).
 */
export function MapSelect({ maps, value, onChange }: Props) {
  const t = useTranslations("Match.form");
  const items: (Map | null)[] = [
    ...maps.filter((m) => m.duel_pool),
    ...maps.filter((m) => !m.duel_pool),
  ];
  const option = (m: Map | null) =>
    m ? (
      <span className={styles.option}>
        <span className={styles.thumbnail}>
          <MapThumbnail slug={m.slug} />
        </span>
        {m.name}
      </span>
    ) : (
      ""
    );

  return (
    <Dropdown<Map | null>
      titleText={<RequiredLabel required>{t("map")}</RequiredLabel>}
      label={t("chooseMap")}
      items={items}
      itemToString={(m) => m?.name ?? ""}
      itemToElement={option}
      renderSelectedItem={option}
      selectedItem={items.find((m) => m?.slug === value) ?? null}
      onChange={({ selectedItem }) => onChange(selectedItem?.slug ?? "")}
    />
  );
}
