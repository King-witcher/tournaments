import type { ReactNode } from "react";

import styles from "./action-tile.module.scss";
import { Tile } from "./carbon/tile";

type Props = {
  /** Antes do nome, o logo (`Avatar` `square`); decorativo. */
  avatar?: ReactNode;
  /** Sobre o que é a tarefa. */
  title: ReactNode;
  /** O contexto (`MetaLines`, para mais de uma linha). */
  meta?: ReactNode;
  /** O que fazer: um `Button`, `ButtonLink` (`tertiary`, `sm`) ou o botão de uma feature. */
  action: ReactNode;
};

/**
 * Uma tarefa de quem vê, com o nome e o botão que a resolve (no celular, o
 * botão desce): o `Tile` do Carbon como contêiner não interativo, o que ele
 * pede quando há ação dentro (tile que abre uma página é o `LinkTile`). Vai
 * dentro de `TileGrid` `stacked`.
 */
export function ActionTile({ avatar, title, meta, action }: Props) {
  return (
    <Tile className={styles.tile}>
      {avatar}
      <div className={styles.text}>
        <div className={styles.title}>{title}</div>
        {meta && <div className={styles.meta}>{meta}</div>}
      </div>
      <div className={styles.action}>{action}</div>
    </Tile>
  );
}
