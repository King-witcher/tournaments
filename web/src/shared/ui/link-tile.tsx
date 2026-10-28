"use client";

import { ClickableTile } from "@carbon/react";
import type { ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import styles from "./link-tile.module.scss";

type Props = {
  href: Href;
  /** O nome do item. */
  title: ReactNode;
  /** Uma linha de metadado (comunidade, início, papel). */
  meta?: ReactNode;
  /** `StatusTag`s da situação. */
  tags?: ReactNode;
  /** Um link à parte, embaixo do tile (ex.: a próxima partida). */
  footer?: ReactNode;
};

/**
 * Item de lista "minha" que abre uma página: o tile inteiro (`ClickableTile`
 * do Carbon) é o link, com nome, metadado e tags. Vai dentro de `TileGrid`.
 */
export function LinkTile({ href, title, meta, tags, footer }: Props) {
  // O ClickableTile repassa `as` ao Link do Carbon (que o aceita), mas o tipo
  // dele só declara `href` como string.
  const link = { as: IntlLink, href } as unknown as { href: string };
  return (
    <>
      <ClickableTile {...link} className={styles.tile}>
        <div className={styles.body}>
          <div className={styles.title}>{title}</div>
          {meta && <div className={styles.meta}>{meta}</div>}
          {tags && <div className={styles.tags}>{tags}</div>}
        </div>
      </ClickableTile>
      {footer && <div className={styles.footer}>{footer}</div>}
    </>
  );
}
