"use client";

import type { ReactNode } from "react";
import { useState, useTransition } from "react";

import { useImageErrorMessage } from "../hooks/use-image-error-message";
import type { EncodedImage, ImageKind } from "../lib/encode-image";
import { prepareImage } from "../lib/upload-image";
import { ImageInput } from "./image-input";

/** A imagem escolhida e já convertida, com a prévia local (`blob:`). */
export type PendingImage = EncodedImage & { preview: string };

type Props = {
  kind: ImageKind;
  maxBytes: number;
  value: PendingImage | undefined;
  onChange: (image: PendingImage | undefined) => void;
  label?: ReactNode;
  hint?: ReactNode;
};

/**
 * O logo ou a capa de algo que ainda vai ser criado: a imagem é cortada e
 * convertida no navegador e fica só nele, com a prévia, até o formulário
 * criar o dono e mandá-la (`useImageOnCreate`). Desistir do formulário não
 * deixa nada no bucket.
 */
export function ImagePicker({
  kind,
  maxBytes,
  value,
  onChange,
  label,
  hint,
}: Props) {
  const errorMessage = useImageErrorMessage();
  const [pending, startTransition] = useTransition();
  const [sending, setSending] = useState<string>();
  const [error, setError] = useState<string>();

  const replace = (next: EncodedImage | undefined) => {
    if (value) URL.revokeObjectURL(value.preview);
    onChange(next && { ...next, preview: URL.createObjectURL(next.blob) });
  };

  const choose = (file: File) => {
    setSending(file.name);
    startTransition(async () => {
      const result = await prepareImage(file, kind, maxBytes);
      if ("blob" in result) {
        setError(undefined);
        setSending(undefined);
        replace(result);
      } else {
        setError(errorMessage(result));
      }
    });
  };

  return (
    <ImageInput
      kind={kind}
      url={value?.preview}
      label={label}
      hint={hint}
      sending={sending}
      pending={pending}
      error={error}
      onFile={choose}
      onRemove={() => {
        setError(undefined);
        replace(undefined);
      }}
      onDismiss={() => {
        setSending(undefined);
        setError(undefined);
      }}
    />
  );
}
