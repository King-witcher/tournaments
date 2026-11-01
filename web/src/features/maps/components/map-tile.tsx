import { MapThumbnail } from "./map-thumbnail";
import styles from "./map-tile.module.scss";

type Props = {
  slug: string;
  name: string;
  /** Uma linha embaixo do nome (o placar, o estado). */
  detail: string;
};

/**
 * Um mapa como item de grade: a imagem, o nome e uma linha de detalhe. O
 * `<li>` da lista é do chamador, que põe a grade.
 */
export function MapTile({ slug, name, detail }: Props) {
  return (
    <li className={styles.tile}>
      <MapThumbnail slug={slug} />
      <span className={styles.name}>{name}</span>
      <span className={styles.detail}>{detail}</span>
    </li>
  );
}
