import Image from "next/image";
import { getLocale } from "next-intl/server";

import styles from "./product-shot.module.scss";

/**
 * As capturas de tela do site em `public/landing/<nome>-<idioma>-<tema>.webp`,
 * com o tamanho de cada uma (a proporção reserva o lugar antes de carregar).
 * Já saem em WebP e no tamanho certo, sem a otimização do Next (que gastaria
 * a cota da Vercel para nada), como as imagens dos mapas.
 */
const SHOTS = {
  bracket: { width: 1800, height: 1051 },
  veto: { width: 1800, height: 624 },
  result: { width: 1800, height: 1017 },
  ranking: { width: 1800, height: 943 },
};

export type ShotName = keyof typeof SHOTS;

type Props = {
  name: ShotName;
  /** O que a captura mostra, para quem não a vê. */
  alt: string;
};

/**
 * Uma captura de tela, no idioma da página e no tema de quem vê: as duas
 * versões vão na página e o CSS mostra a do tema (a escolha da pessoa fica
 * no `<html>`, que o `<picture>` não lê). A escondida não baixa: o `lazy`
 * só carrega a imagem que aparece. Moldura de 1px para a captura clara não
 * se perder no fundo claro (e a escura no escuro).
 */
export async function ProductShot({ name, alt }: Props) {
  const locale = await getLocale();
  const { width, height } = SHOTS[name];
  const src = (theme: "light" | "dark") =>
    `/landing/${name}-${locale}-${theme}.webp`;

  return (
    <div className={styles.frame}>
      <Image
        src={src("light")}
        alt={alt}
        width={width}
        height={height}
        unoptimized
        className={styles.light}
      />
      <Image
        src={src("dark")}
        alt={alt}
        width={width}
        height={height}
        unoptimized
        className={styles.dark}
      />
    </div>
  );
}
