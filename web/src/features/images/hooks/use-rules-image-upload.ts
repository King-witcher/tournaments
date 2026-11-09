"use client";

import type { MarkdownImageUpload } from "@/shared/ui/markdown-field";
import type { RulesImages } from "../lib/rules-images";
import { uploadRulesImage } from "../lib/upload-image";
import { useImageErrorMessage } from "./use-image-error-message";

/**
 * O envio de uma imagem das regras de um torneio para o `onImage` do
 * `MarkdownField`: reduzida no navegador, mandada ao bucket e registrada na
 * comunidade; devolve a URL do CDN ou o erro traduzido. Sem `images` (sem o
 * bucket público), `undefined`, e o botão não aparece.
 */
export function useRulesImageUpload(
  tenant: string,
  tournament: string,
  images: RulesImages | null | undefined,
): MarkdownImageUpload | undefined {
  const errorMessage = useImageErrorMessage();
  if (!images) return undefined;
  return async (file) => {
    const result = await uploadRulesImage(
      { kind: "rules", tenant, tournament },
      file,
      images.maxBytes,
    );
    return result.ok
      ? { ok: true, url: result.url }
      : { ok: false, error: errorMessage(result) };
  };
}
