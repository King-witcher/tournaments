import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { PrintTarget } from "../lib/print-target";
import styles from "./print-thumbnail.module.scss";
import { RemovePrintButton } from "./remove-print-button";

type Props = {
  target: PrintTarget;
  print: components["schemas"]["Print"];
  /** Posição no mapa, a partir de 1 (para o nome acessível). */
  index: number;
  focusId: string;
};

/**
 * Um print: a miniatura é o link para a imagem em tamanho real (numa aba
 * nova, a URL assinada que a API deu), com quem anexou embaixo e "Remover"
 * para quem pode.
 */
export function PrintThumbnail({ target, print, index, focusId }: Props) {
  const t = useTranslations("Prints");
  const name = print.uploaded_by?.nickname;
  const number = target.number;
  const label = name
    ? t("open", { index, number, name })
    : t("openNoAuthor", { index, number });

  return (
    <figure className={styles.figure}>
      <a
        href={print.url}
        target="_blank"
        rel="noopener noreferrer"
        className={styles.link}
      >
        {/* biome-ignore lint/performance/noImgElement: a URL é assinada e muda a cada hora; o next/image passaria cada print pelo servidor do Next e pediria o domínio do bucket no config. */}
        <img
          src={print.url}
          alt={label}
          loading="lazy"
          decoding="async"
          className={styles.image}
        />
      </a>
      {name && (
        <figcaption className={styles.caption}>{t("by", { name })}</figcaption>
      )}
      {print.can_remove && (
        <RemovePrintButton
          target={target}
          printId={print.id}
          index={index}
          focusId={focusId}
        />
      )}
    </figure>
  );
}
