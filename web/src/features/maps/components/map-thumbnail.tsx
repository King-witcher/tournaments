import Image from "next/image";

import { cn } from "@/shared/lib/cn";
import styles from "./map-thumbnail.module.scss";

/**
 * Os mapas com imagem em `public/maps/<slug>.webp`: a tela de carregamento do
 * jogo, recortada em 16:9, em 480×270. Já sai em WebP e no tamanho em que
 * aparece, então não passa pela otimização do Next (que gastaria a cota da
 * Vercel para nada).
 */
const IMAGES = new Set([
  "awoken",
  "blood-covenant",
  "blood-run",
  "burial-chamber",
  "church-of-azathoth",
  "citadel",
  "corrupted-keep",
  "crucible",
  "cursed-soul",
  "dark-zone",
  "deep-embrace",
  "exile",
  "insomnia",
  "lockbox",
  "molten-falls",
  "ruins-of-sarnath",
  "tempest-shrine",
  "the-longest-yard",
  "tower-of-koth",
  "vale-of-pnath",
]);

type Props = {
  slug: string;
  /** Fora (banido no veto, ou fora do pool no formulário): em tons de cinza. */
  out?: boolean;
};

/**
 * A imagem do mapa, na largura de quem a contém, em 16:9; decorativa (o nome
 * sempre aparece ao lado). Mapa sem imagem fica com o fundo, do mesmo
 * tamanho, para a grade não desalinhar.
 */
export function MapThumbnail({ slug, out = false }: Props) {
  const className = cn(styles.thumbnail, out && styles.out);
  if (!IMAGES.has(slug)) return <span aria-hidden className={className} />;
  return (
    <Image
      src={`/maps/${slug}.webp`}
      alt=""
      width={480}
      height={270}
      unoptimized
      className={className}
    />
  );
}
