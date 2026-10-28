"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Select } from "@/shared/ui/select";

type Map = components["schemas"]["Map"];

type Props = {
  maps: Map[];
  value: string;
  onChange: (slug: string) => void;
};

/** Os mapas ativos, o pool de duelo primeiro. */
export function MapSelect({ maps, value, onChange }: Props) {
  const t = useTranslations("Match.form");
  const option = (m: Map) => (
    <option key={m.slug} value={m.slug}>
      {m.name}
    </option>
  );

  return (
    <Select value={value} onChange={(e) => onChange(e.target.value)}>
      <option value="">{t("chooseMap")}</option>
      <optgroup label={t("pool")}>
        {maps.filter((m) => m.duel_pool).map(option)}
      </optgroup>
      <optgroup label={t("others")}>
        {maps.filter((m) => !m.duel_pool).map(option)}
      </optgroup>
    </Select>
  );
}
