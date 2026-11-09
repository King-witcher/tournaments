import Image from "next/image";

import { Avatar } from "@/shared/ui/avatar";
import styles from "./tournament-media.module.scss";

type Props = {
  /** A capa do torneio. */
  cover?: string | null;
  /** A capa da comunidade, no lugar da que falta. */
  tenantCover?: string | null;
  /** O logo da comunidade, sem nenhuma das duas capas. */
  logo?: string | null;
  /** O nome da comunidade, para a inicial quando também falta o logo. */
  community: string;
};

/**
 * O que vai no topo do tile do torneio (`LinkTile` `media`): a capa dele ou a
 * da comunidade; sem nenhuma, o logo da comunidade (ou a inicial) no centro do
 * mesmo espaço, para a grade não desalinhar. Decorativo: o nome vem embaixo.
 */
export function TournamentMedia({
  cover,
  tenantCover,
  logo,
  community,
}: Props) {
  const image = cover ?? tenantCover;
  if (image) {
    return (
      <Image src={image} alt="" fill unoptimized className={styles.cover} />
    );
  }
  return (
    <div className={styles.fallback}>
      <Avatar shape="square" size="lg" src={logo} name={community} />
    </div>
  );
}
