import { type ApiErrorParams, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import {
  type EncodedImage,
  type EncodeError,
  encodeImage,
  encodeScaled,
} from "./encode-image";

/**
 * De quem é a imagem: o logo da comunidade, a capa de um torneio dela, a
 * foto de perfil de quem está logado ou uma imagem das regras de um torneio
 * (que entra no Markdown pela URL devolvida).
 */
export type ImageTarget =
  | { kind: "logo"; tenant: string }
  | { kind: "cover"; tenant: string; tournament: string }
  | { kind: "avatar" }
  | { kind: "rules"; tenant: string; tournament: string };

/** O logo, a capa ou a foto: uma imagem por dono, que se troca e se tira. */
export type FixedImageTarget = Exclude<ImageTarget, { kind: "rules" }>;

/** Erro do navegador, fora da API (o texto fica em `Images.errors`). */
export type LocalError = EncodeError | "notImage" | "uploadFailed";

export type ImageOutcome =
  | { ok: true; url: string }
  | { ok: false; local: LocalError }
  | { ok: false; code: string; params?: ApiErrorParams };

type NewUpload = components["schemas"]["NewImageUpload"];

function requestUpload(target: ImageTarget, body: NewUpload) {
  switch (target.kind) {
    case "logo":
      return api.POST("/tenants/{slug}/logo/uploads", {
        params: { path: { slug: target.tenant } },
        body,
      });
    case "cover":
      return api.POST(
        "/tenants/{slug}/tournaments/{tournament}/cover/uploads",
        {
          params: {
            path: { slug: target.tenant, tournament: target.tournament },
          },
          body,
        },
      );
    case "avatar":
      return api.POST("/me/avatar/uploads", { body });
    case "rules":
      return api.POST(
        "/tenants/{slug}/tournaments/{tournament}/rules/images/uploads",
        {
          params: {
            path: { slug: target.tenant, tournament: target.tournament },
          },
          body,
        },
      );
  }
}

function attach(target: ImageTarget, upload: number) {
  switch (target.kind) {
    case "logo":
      return api.POST("/tenants/{slug}/logo/uploads/{upload}/attach", {
        params: { path: { slug: target.tenant, upload } },
      });
    case "cover":
      return api.POST(
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
    case "avatar":
      return api.POST("/me/avatar/uploads/{upload}/attach", {
        params: { path: { upload } },
      });
    case "rules":
      return api.POST(
        "/tenants/{slug}/tournaments/{tournament}/rules/images/uploads/{upload}/attach",
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
}

/** Tira o logo, a capa ou a foto. */
export function removeImage(target: FixedImageTarget) {
  switch (target.kind) {
    case "logo":
      return api.DELETE("/tenants/{slug}/logo", {
        params: { path: { slug: target.tenant } },
      });
    case "cover":
      return api.DELETE("/tenants/{slug}/tournaments/{tournament}/cover", {
        params: {
          path: { slug: target.tenant, tournament: target.tournament },
        },
      });
    case "avatar":
      return api.DELETE("/me/avatar");
  }
}

/**
 * O caminho inteiro de um logo, de uma capa ou de uma imagem das regras:
 * converte no navegador (`prepareImage`) e manda (`sendImage`).
 */
export async function uploadImage(
  target: ImageTarget,
  file: Blob,
  maxBytes: number,
): Promise<ImageOutcome> {
  const encoded = await prepareImage(file, target.kind, maxBytes);
  return "blob" in encoded ? sendImage(target, encoded) : encoded;
}

/**
 * Converte no navegador, sem mandar nada: logo e capa cortados pelo centro
 * na proporção deles; a imagem das regras só reduzida.
 */
async function prepareImage(
  file: Blob,
  kind: ImageTarget["kind"],
  maxBytes: number,
): Promise<EncodedImage | { ok: false; local: LocalError }> {
  if (!file.type.startsWith("image/")) return { ok: false, local: "notImage" };
  const encoded =
    kind === "rules"
      ? await encodeScaled(file, maxBytes)
      : await encodeImage(file, kind, maxBytes);
  return typeof encoded === "string" ? { ok: false, local: encoded } : encoded;
}

/**
 * Manda uma imagem já convertida: pede a URL de upload à API, manda o
 * arquivo direto ao bucket (PUT, com o tipo, o `Cache-Control` e o
 * `If-None-Match: *` que a URL assinou: o bucket grava uma vez só) e
 * registra na API, que confere o que chegou e só então troca a imagem.
 * Parado no meio (erro, aba fechada), a imagem de antes continua, o upload
 * fica pendente e a rotina diária da API apaga o objeto.
 */
export async function sendImage(
  target: ImageTarget,
  encoded: EncodedImage,
): Promise<ImageOutcome> {
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
  return attached.ok ? { ok: true, url: attached.data.url } : attached;
}
