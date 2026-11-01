import Image from "next/image";

import { Avatar } from "@/shared/ui/avatar";
import styles from "./tournament-media.module.scss";

type Props = {
  /** A capa do torneio. */
  cover?: string | null;
  /** O logo da comunidade, no lugar da capa que falta. */
  logo?: string | null;
  /** O nome da comunidade, para a inicial quando também falta o logo. */
  community: string;
};

/**
 * O que vai no topo do tile do torneio (`LinkTile` `media`): a capa ou, sem
 * ela, o logo da comunidade (ou a inicial) no centro do mesmo espaço, para a
 * grade não desalinhar. Decorativo: o nome vem embaixo.
 */
export function TournamentMedia({ cover, logo, community }: Props) {
  if (cover) {
    return (
      <Image src={cover} alt="" fill unoptimized className={styles.cover} />
    );
  }
  return (
    <div className={styles.fallback}>
      <Avatar shape="square" size="lg" src={logo} name={community} />
    </div>
  );
}
