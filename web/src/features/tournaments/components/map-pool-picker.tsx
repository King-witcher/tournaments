import { useId, useState } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { rememberPool } from "../lib/remembered-pool";
import { MapPoolGrid } from "./map-pool-grid";

type Props = {
  tenant: string;
  maps: components["schemas"]["Map"][];
  /** O pool de agora. */
  pool: string[];
  /** Quantos mapas a série mais longa pede: faltando, os de fora abrem. */
  min: number;
  hasError: boolean;
  readOnly: boolean;
  /** Ver `MapPoolField`. */
  savedPool?: string[];
  onChange: (mapPool: string[]) => void;
};

/**
 * A grade do Personalizado (e do só leitura), que existe só enquanto a pessoa
 * escolhe mapa a mapa: sem `savedPool`, o pool de quando ela apareceu divide
 * os mapas. Cada marca ou desmarca fica lembrada neste navegador. Os mapas de
 * fora começam atrás de um botão (eram dois terços do grupo aberto) e aparecem
 * sozinhos quando o pool não basta ou quando algum deles está marcado.
 */
export function MapPoolPicker({
  tenant,
  maps,
  pool,
  min,
  hasError,
  readOnly,
  savedPool,
  onChange,
}: Props) {
  const outHeadingId = useId();
  const [outOpen, setOutOpen] = useState(false);
  const [initialPool] = useState(pool);
  const base = savedPool ?? initialPool;
  const outShown =
    outOpen ||
    pool.length < min ||
    hasError ||
    pool.some((s) => !base.includes(s));

  const toggle = (slug: string, on: boolean) => {
    const mapPool = on
      ? maps
          .filter((m) => pool.includes(m.slug) || m.slug === slug)
          .map((m) => m.slug)
      : pool.filter((s) => s !== slug);
    onChange(mapPool);
    rememberPool(tenant, mapPool);
  };
  const showOut = () => {
    setOutOpen(true);
    // O botão some: o foco vai para o subtítulo dos de fora.
    focusIfLost(outHeadingId);
  };

  return (
    <MapPoolGrid
      maps={maps}
      pool={pool}
      base={base}
      readOnly={readOnly}
      onToggle={toggle}
      outHeadingId={outHeadingId}
      outShown={outShown}
      onShowOut={showOut}
    />
  );
}
