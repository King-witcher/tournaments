import type { components } from "@/shared/api/schema";

/** O que a página sabe das imagens das regras (`GET /images/settings`). */
export type RulesImages = { prefix: string; maxBytes: number };

/**
 * As imagens das regras a partir de `GET /images/settings`: o começo da URL
 * que vale e o tamanho máximo do arquivo; `null` sem o bucket público (nem
 * botão, nem imagem desenhada).
 */
export function rulesImagesOf(
  settings: components["schemas"]["ImageSettings"] | null,
): RulesImages | null {
  const prefix = settings?.rules_image_prefix;
  return settings && prefix ? { prefix, maxBytes: settings.max_bytes } : null;
}
