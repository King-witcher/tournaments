"use client";

import { useState, useTransition } from "react";

import { useRouter } from "@/i18n/navigation";
import { useApiAction } from "@/shared/api/use-api-action";
import { useImageErrorMessage } from "../hooks/use-image-error-message";
import {
  type FixedImageTarget,
  removeImage,
  uploadImage,
} from "../lib/upload-image";
import { ImageInput } from "./image-input";

type Props = {
  target: FixedImageTarget;
  /** A imagem de agora; nula sem imagem. */
  url: string | null | undefined;
  maxBytes: number;
};

/**
 * O logo ou a capa que já existem, que se salvam sozinhos (fora do "Salvar"
 * do formulário). A imagem é cortada e convertida no navegador, vai direto
 * ao bucket e é registrada na API; o erro sai traduzido junto do controle.
 */
export function ImageField({ target, url, maxBytes }: Props) {
  const errorMessage = useImageErrorMessage();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const [sending, setSending] = useState<string>();
  const [error, setError] = useState<string>();
  const removal = useApiAction();

  const send = (file: File) => {
    setSending(file.name);
    startTransition(async () => {
      const result = await uploadImage(target, file, maxBytes);
      if (result.ok) {
        setError(undefined);
        setSending(undefined);
        router.refresh();
      } else {
        setError(errorMessage(result));
      }
    });
  };

  return (
    <ImageInput
      kind={target.kind}
      url={url}
      sending={sending}
      pending={pending}
      error={error ?? removal.error}
      removing={removal.pending}
      onFile={send}
      onRemove={() =>
        removal.run(
          () => removeImage(target),
          () => router.refresh(),
        )
      }
      onDismiss={() => {
        setSending(undefined);
        setError(undefined);
      }}
    />
  );
}
