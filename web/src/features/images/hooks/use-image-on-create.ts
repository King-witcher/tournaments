"use client";

import { useState } from "react";

import type { PendingImage } from "../components/image-picker";
import {
  type ImageOutcome,
  type ImageTarget,
  sendImage,
} from "../lib/upload-image";

/** A imagem que não chegou ao que acabou de ser criado. */
export type UnsentImage = {
  target: ImageTarget;
  image: PendingImage;
  failure: Exclude<ImageOutcome, { ok: true }>;
};

/**
 * O logo ou a capa escolhidos num formulário de criação. Nada vai ao bucket
 * antes de o dono existir: `send`, chamado depois de criar, manda a imagem
 * guardada no navegador. Falhou, ela fica em `unsent`, para a pessoa tentar
 * de novo ou seguir sem ela (`ImageNotSent`).
 */
export function useImageOnCreate() {
  const [image, setImage] = useState<PendingImage>();
  const [unsent, setUnsent] = useState<UnsentImage>();

  /** Manda a imagem escolhida, se há; falso se ela não chegou. */
  const send = async (target: ImageTarget) => {
    if (!image) return true;
    const sent = await sendImage(target, image);
    if (sent.ok) return true;
    setUnsent({ target, image, failure: sent });
    return false;
  };

  return { image, setImage, send, unsent };
}
