"use client";

import { ArrowRight } from "@carbon/icons-react";
import type { ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import { AspectRatio } from "./carbon/aspect-ratio";
import { ClickableTile } from "./carbon/tile";
import styles from "./link-tile.module.scss";

type Props = {
  href: Href;
  /** O nome do item. */
  title: ReactNode;
  /** Antes do nome, o logo (`Avatar` `square`); decorativo. */
  avatar?: ReactNode;
  /**
   * No topo do tile, em 2:1: a capa ou o que a substitui. Na grade, todos os
   * tiles levam ou nenhum leva, para as linhas não desalinharem.
   */
  media?: ReactNode;
  /** Uma linha de metadado (comunidade, início, papel). */
  meta?: ReactNode;
  /** `StatusTag`s da situação. */
  tags?: ReactNode;
  /** Ao abrir, além de navegar. */
  onClick?: () => void;
};

/**
 * Item de lista "minha" que abre uma página: o tile inteiro (`ClickableTile`
 * do Carbon) é o link, com a capa (opcional, no topo), o logo e o nome,
 * metadado, tags e a seta no canto de baixo, que o Carbon põe no tile que
 * navega (sem hover no celular, é ela que diz que o tile abre algo). Vai
 * dentro de `TileGrid`.
 */
export function LinkTile({
  href,
  title,
  avatar,
  media,
  meta,
  tags,
  onClick,
}: Props) {
  // O ClickableTile repassa `as` ao Link do Carbon (que o aceita), mas o tipo
  // dele só declara `href` como string.
  const link = { as: IntlLink, href } as unknown as { href: string };
  return (
    <ClickableTile
      {...link}
      className={styles.tile}
      renderIcon={ArrowRight}
      onClick={onClick}
    >
      {media && (
        <AspectRatio ratio="2x1" className={styles.media}>
          {media}
        </AspectRatio>
      )}
      <div className={styles.body}>
        <div className={styles.name}>
          {avatar}
          <div className={styles.title}>{title}</div>
        </div>
        {meta && <div className={styles.meta}>{meta}</div>}
        {tags && <div className={styles.tags}>{tags}</div>}
      </div>
    </ClickableTile>
  );
}
