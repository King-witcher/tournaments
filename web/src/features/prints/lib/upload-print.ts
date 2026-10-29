import { type ApiErrorParams, callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { type EncodeError, encodePrint } from "./encode-print";
import { type PrintTarget, printParams } from "./print-target";

/** Erro do navegador, fora da API (o texto fica em `Prints.errors`). */
export type LocalError = EncodeError | "notImage" | "uploadFailed";

export type UploadOutcome =
  | { ok: true }
  | { ok: false; local: LocalError }
  | { ok: false; code: string; params?: ApiErrorParams };

/**
 * O caminho inteiro de um print: converte no navegador, pede a URL de upload
 * à API, manda o arquivo direto ao bucket (PUT, com o tipo que a URL
 * assinou) e registra o print na API, que confere o que chegou.
 */
export async function uploadPrint(
  target: PrintTarget,
  file: Blob,
  maxBytes: number,
): Promise<UploadOutcome> {
  if (!file.type.startsWith("image/")) return { ok: false, local: "notImage" };
  const encoded = await encodePrint(file, maxBytes);
  if (typeof encoded === "string") return { ok: false, local: encoded };

  const params = printParams(target);
  const upload = await callApi(
    api.POST(
      "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/prints",
      {
        params,
        body: { content_type: encoded.type, size: encoded.blob.size },
      },
    ),
  );
  if (!upload.ok) return upload;
  const sent = await fetch(upload.data.url, {
    method: "PUT",
    headers: { "Content-Type": upload.data.content_type },
    body: encoded.blob,
  }).catch(() => undefined);
  if (!sent?.ok) return { ok: false, local: "uploadFailed" };

  const attached = await callApi(
    api.POST(
      "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/prints/{print}/attach",
      { params: { path: { ...params.path, print: upload.data.id } } },
    ),
  );
  return attached.ok ? { ok: true } : attached;
}
