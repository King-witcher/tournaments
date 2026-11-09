"use client";

import {
  startTransition,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";

import { callApi } from "@/shared/api/call";
import type { EncodedImage } from "../lib/encode-image";
import {
  type FixedImageTarget,
  type ImageOutcome,
  removeImage,
  sendImage,
} from "../lib/upload-image";

/** O que falta mandar: a imagem recortada, com a prévia dela, ou tirar a de agora. */
type PendingImage =
  | { action: "replace"; encoded: EncodedImage; preview: string }
  | { action: "remove" };

export type ImageFailure = Exclude<ImageOutcome, { ok: true }>;

/**
 * A imagem de um formulário enquanto o "Salvar" dele não chega: trocar
 * (`replace`, com o recorte já codificado) e tirar (`remove`) só ficam aqui,
 * como o valor editado de um campo, e `apply`, chamado pelo Salvar, é o único
 * que fala com a API e com o bucket. Quem chama fica com o estado e o liga ao
 * próprio Salvar e à própria guarda de saída (`dirty`); o `ImageField` só
 * mostra. `apply` devolve se nada ficou por mandar: no erro, a pendência fica,
 * com o erro em `failure`, para o próximo Salvar repetir só ela; no acerto,
 * ela sai numa transition, para a tela trocar a prévia pela imagem salva de
 * uma vez, junto do `router.refresh()` que quem chamou faz. O objeto muda só
 * quando o estado muda.
 */
export function useImageDraft(target: FixedImageTarget) {
  const [pending, setPending] = useState<PendingImage>();
  const [failure, setFailure] = useState<ImageFailure>();
  const preview = pending?.action === "replace" ? pending.preview : undefined;

  // Depois da troca na tela: a prévia de antes ainda aparece até o commit.
  useEffect(() => {
    if (!preview) return;
    return () => URL.revokeObjectURL(preview);
  }, [preview]);

  const replace = useCallback((encoded: EncodedImage) => {
    setFailure(undefined);
    setPending({
      action: "replace",
      encoded,
      preview: URL.createObjectURL(encoded.blob),
    });
  }, []);

  const remove = useCallback(() => {
    setFailure(undefined);
    setPending({ action: "remove" });
  }, []);

  const discard = useCallback(() => {
    setFailure(undefined);
    setPending(undefined);
  }, []);

  const apply = useCallback(async () => {
    if (!pending) return true;
    setFailure(undefined);
    const outcome =
      pending.action === "replace"
        ? await sendImage(target, pending.encoded)
        : await callApi(removeImage(target));
    if (!outcome.ok) {
      setFailure(outcome);
      return false;
    }
    startTransition(() => setPending(undefined));
    return true;
  }, [pending, target]);

  return useMemo(
    () => ({
      target,
      /** Tem troca ou remoção esperando o Salvar. */
      dirty: pending !== undefined,
      /** A prévia da imagem pendente. */
      preview,
      /** A imagem de agora vai sair ao salvar. */
      removing: pending?.action === "remove",
      /** Por que o último Salvar não mandou a imagem. */
      failure,
      replace,
      remove,
      discard,
      apply,
    }),
    [target, pending, preview, failure, replace, remove, discard, apply],
  );
}

export type ImageDraft = ReturnType<typeof useImageDraft>;
