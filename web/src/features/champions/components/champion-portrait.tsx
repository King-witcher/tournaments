import Image from "next/image";

import { cn } from "@/shared/lib/cn";
import { Tooltip } from "@/shared/ui/tooltip";
import styles from "./champion-portrait.module.scss";

/**
 * Os campeões com retrato em `public/champions/<forma>/<slug>.webp`, tirados
 * da interface do jogo (`roster`: o retrato 3:4 da escolha de campeão, 86×123;
 * `big`: o quadrado com vinheta, 250×250). Já saem em WebP e no tamanho em
 * que aparecem, então não passam pela otimização do Next.
 */
const IMAGES = new Set([
  "anarki",
  "athena",
  "bj-blazkowicz",
  "clutch",
  "death-knight",
  "doom-slayer",
  "eisen",
  "galena",
  "keel",
  "nyx",
  "ranger",
  "scalebearer",
  "slash",
  "sorlag",
  "strogg-peeker",
  "visor",
]);

const SIZES = {
  roster: { width: 86, height: 123 },
  big: { width: 250, height: 250 },
};

type Props = {
  slug: string;
  /**
   * O nome, na dica ao passar o mouse ou focar: o retrato passa a ser o único
   * lugar dele. Sem isto, o retrato é decorativo e quem o contém dá o nome
   * (o tile da grade do draft).
   */
  name?: string;
  /** `roster` (padrão): 3:4, para o draft e as miniaturas; `big`: quadrado, para o cartão da partida. */
  shape?: keyof typeof SIZES;
  /** Fora (banido no draft): em tons de cinza. */
  out?: boolean;
};

/**
 * O retrato do campeão, na largura de quem o contém. Campeão sem retrato fica
 * com o fundo, do mesmo tamanho, para a grade não desalinhar. Não arrasta,
 * como a imagem do mapa: fica dentro do que se clica.
 */
export function ChampionPortrait({
  slug,
  name,
  shape = "roster",
  out = false,
}: Props) {
  const className = cn(styles.portrait, styles[shape], out && styles.out);
  const portrait = IMAGES.has(slug) ? (
    <Image
      src={`/champions/${shape}/${slug}.webp`}
      alt=""
      {...SIZES[shape]}
      unoptimized
      draggable={false}
      className={className}
    />
  ) : (
    <span aria-hidden className={className} />
  );
  if (!name) return portrait;
  return (
    <Tooltip label={name} className={styles.tip}>
      {/* biome-ignore lint/a11y/noNoninteractiveTabindex: o nome só existe na dica, que precisa abrir também pelo teclado. */}
      <span role="img" tabIndex={0} className={styles.trigger}>
        {portrait}
      </span>
    </Tooltip>
  );
}
