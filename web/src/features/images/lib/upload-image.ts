import { type ApiErrorParams, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { type EncodeError, encodeImage } from "./encode-image";

/** De quem é a imagem: o logo da comunidade ou a capa de um torneio dela. */
export type ImageTarget =
  | { kind: "logo"; tenant: string }
  | { kind: "cover"; tenant: string; tournament: string };

/** Erro do navegador, fora da API (o texto fica em `Images.errors`). */
export type LocalError = EncodeError | "notImage" | "uploadFailed";

export type ImageOutcome =
  | { ok: true }
  | { ok: false; local: LocalError }
  | { ok: false; code: string; params?: ApiErrorParams };

type NewUpload = components["schemas"]["NewImageUpload"];

function requestUpload(target: ImageTarget, body: NewUpload) {
  return target.kind === "logo"
    ? api.POST("/tenants/{slug}/logo/uploads", {
        params: { path: { slug: target.tenant } },
        body,
      })
    : api.POST("/tenants/{slug}/tournaments/{tournament}/cover/uploads", {
        params: {
          path: { slug: target.tenant, tournament: target.tournament },
        },
        body,
      });
}

function attach(target: ImageTarget, upload: number) {
  return target.kind === "logo"
    ? api.POST("/tenants/{slug}/logo/uploads/{upload}/attach", {
        params: { path: { slug: target.tenant, upload } },
      })
    : api.POST(
        "/tenants/{slug}/tournaments/{tournament}/cover/uploads/{upload}/attach",
        {
          params: {
            path: {
              slug: target.tenant,
              tournament: target.tournament,
              upload,
            },
          },
        },
      );
}

/** Tira o logo ou a capa. */
export function removeImage(target: ImageTarget) {
  return target.kind === "logo"
    ? api.DELETE("/tenants/{slug}/logo", {
        params: { path: { slug: target.tenant } },
      })
    : api.DELETE("/tenants/{slug}/tournaments/{tournament}/cover", {
        params: {
          path: { slug: target.tenant, tournament: target.tournament },
        },
      });
}

/**
 * O caminho inteiro de um logo ou de uma capa: corta e converte no
 * navegador, pede a URL de upload à API, manda o arquivo direto ao bucket
 * (PUT, com o tipo, o `Cache-Control` e o `If-None-Match: *` que a URL
 * assinou: o bucket grava uma vez só) e registra na API, que confere o que
 * chegou e troca a imagem.
 */
export async function uploadImage(
  target: ImageTarget,
  file: Blob,
  maxBytes: number,
): Promise<ImageOutcome> {
  if (!file.type.startsWith("image/")) return { ok: false, local: "notImage" };
  const encoded = await encodeImage(file, target.kind, maxBytes);
  if (typeof encoded === "string") return { ok: false, local: encoded };

  const body = { content_type: encoded.type, size: encoded.blob.size };
  const upload = await callApi(requestUpload(target, body));
  if (!upload.ok) return upload;
  const sent = await fetch(upload.data.url, {
    method: "PUT",
    headers: {
      "Content-Type": upload.data.content_type,
      "Cache-Control": upload.data.cache_control,
      "If-None-Match": "*",
    },
    body: encoded.blob,
  }).catch(() => undefined);
  if (!sent?.ok) return { ok: false, local: "uploadFailed" };

  const attached = await callApi(attach(target, upload.data.id));
  return attached.ok ? { ok: true } : attached;
}
