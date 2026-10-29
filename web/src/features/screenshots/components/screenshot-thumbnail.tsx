import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { ScreenshotTarget } from "../lib/screenshot-target";
import { RemoveScreenshotButton } from "./remove-screenshot-button";
import styles from "./screenshot-thumbnail.module.scss";

type Props = {
  target: ScreenshotTarget;
  screenshot: components["schemas"]["Screenshot"];
  /** Posição no mapa, a partir de 1 (para o nome acessível). */
  index: number;
  focusId: string;
};

/**
 * Um screenshot: a miniatura é o link para a imagem em tamanho real (numa aba
 * nova, a URL assinada que a API deu), com quem anexou embaixo e "Remover"
 * para quem pode.
 */
export function ScreenshotThumbnail({
  target,
  screenshot,
  index,
  focusId,
}: Props) {
  const t = useTranslations("Screenshots");
  const name = screenshot.uploaded_by?.nickname;
  const number = target.number;
  const label = name
    ? t("open", { index, number, name })
    : t("openNoAuthor", { index, number });

  return (
    <figure className={styles.figure}>
      <a
        href={screenshot.url}
        target="_blank"
        rel="noopener noreferrer"
        className={styles.link}
      >
        {/* biome-ignore lint/performance/noImgElement: a URL é assinada e muda a cada hora; o next/image passaria cada screenshot pelo servidor do Next e pediria o domínio do bucket no config. */}
        <img
          src={screenshot.url}
          alt={label}
          loading="lazy"
          decoding="async"
          className={styles.image}
        />
      </a>
      {name && (
        <figcaption className={styles.caption}>{t("by", { name })}</figcaption>
      )}
      {screenshot.can_remove && (
        <RemoveScreenshotButton
          target={target}
          screenshotId={screenshot.id}
          index={index}
          focusId={focusId}
        />
      )}
    </figure>
  );
}
