import type { ReactNode } from "react";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import styles from "./veto-map-tile.module.scss";

type Props = {
  map: components["schemas"]["MapRef"];
  /** O que aconteceu com ele no veto, ou os votos. */
  state: string;
  /** Banido: imagem em cinza e nome riscado. */
  out?: boolean;
  /** Na vez de quem vê, o botão de banir, escolher ou votar. */
  children?: ReactNode;
};

/**
 * Um mapa no veto ou na votação: a imagem, o nome, o que aconteceu com ele e,
 * embaixo, o botão.
 */
export function VetoMapTile({ map, state, out = false, children }: Props) {
  return (
    <li className={cn(styles.tile, out && styles.out)}>
      <MapThumbnail slug={map.slug} out={out} />
      <span className={styles.name}>{map.name}</span>
      <span className={styles.state}>{state}</span>
      {children}
    </li>
  );
}
