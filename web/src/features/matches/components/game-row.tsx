"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Select } from "@/shared/ui/select";
import type { GameValues } from "../lib/match-target";

type Props = {
  /** Posição na série, a partir de 1. */
  number: number;
  game: GameValues;
  onChange: (game: GameValues) => void;
  onRemove: () => void;
  maps: components["schemas"]["Map"][];
  names: { a: string; b: string };
};

/** Um mapa do detalhe: qual (pool de duelo primeiro) e os frags de cada lado. */
export function GameRow({
  number,
  game,
  onChange,
  onRemove,
  maps,
  names,
}: Props) {
  const t = useTranslations("Match.report");
  const option = (m: components["schemas"]["Map"]) => (
    <option key={m.slug} value={m.slug}>
      {m.name}
    </option>
  );
  const frags = (side: "a" | "b") => (
    <Input
      type="number"
      min={0}
      max={999}
      inputMode="numeric"
      aria-label={t("frags", { name: names[side] })}
      placeholder={names[side]}
      value={game[side]}
      onChange={(e) => onChange({ ...game, [side]: e.target.value })}
      className="w-20"
    />
  );

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Select
        aria-label={t("map", { number })}
        value={game.map}
        onChange={(e) => onChange({ ...game, map: e.target.value })}
        className="w-48"
      >
        <option value="">{t("chooseMap")}</option>
        <optgroup label={t("pool")}>
          {maps.filter((m) => m.duel_pool).map(option)}
        </optgroup>
        <optgroup label={t("others")}>
          {maps.filter((m) => !m.duel_pool).map(option)}
        </optgroup>
      </Select>
      {frags("a")}
      {frags("b")}
      <Button
        size="sm"
        variant="link"
        onClick={onRemove}
        aria-label={t("removeGame", { number })}
      >
        {t("remove")}
      </Button>
    </div>
  );
}
