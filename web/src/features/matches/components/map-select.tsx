"use client";

import { SelectItem, SelectItemGroup } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Select } from "@/shared/ui/select";

type Map = components["schemas"]["Map"];

type Props = {
  maps: Map[];
  value: string;
  onChange: (slug: string) => void;
  /** Erro do campo, já traduzido. */
  invalidText?: string;
};

/** O mapa jogado: os ativos, o pool de duelo primeiro. */
export function MapSelect({ maps, value, onChange, invalidText }: Props) {
  const t = useTranslations("Match.form");
  const item = (m: Map) => (
    <SelectItem key={m.slug} value={m.slug} text={m.name} />
  );

  return (
    <Select
      labelText={t("map")}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      invalid={invalidText !== undefined}
      invalidText={invalidText}
    >
      <SelectItem value="" text={t("chooseMap")} />
      <SelectItemGroup label={t("pool")}>
        {maps.filter((m) => m.duel_pool).map(item)}
      </SelectItemGroup>
      <SelectItemGroup label={t("others")}>
        {maps.filter((m) => !m.duel_pool).map(item)}
      </SelectItemGroup>
    </Select>
  );
}
