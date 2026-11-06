"use client";

import { useId, useState, useTransition } from "react";

import { useRouter } from "@/i18n/navigation";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { useImageErrorMessage } from "../hooks/use-image-error-message";
import { type LocalError, removeImage } from "../lib/upload-image";
import { AvatarEditor } from "./avatar-editor";
import { ImageInput } from "./image-input";

type Props = {
  /** A foto enviada de agora; nula sem ela (mesmo com o avatar do Discord). */
  url: string | null | undefined;
  maxBytes: number;
};

/**
 * A foto de perfil, na aba Perfil da conta: escolher o arquivo abre o editor
 * (`AvatarEditor`), que recorta, gira e só então envia; cancelar não manda
 * nada. "Remover" tira a foto na hora (volta o avatar do Discord, se houver).
 * O arquivo que não dá para ler fica com o erro na área de soltar.
 */
export function AvatarField({ url, maxBytes }: Props) {
  const errorMessage = useImageErrorMessage();
  const router = useRouter();
  const id = useId();
  const [image, setImage] = useState<ImageBitmap>();
  const [reading, startReading] = useTransition();
  const [failed, setFailed] = useState<{ name: string; error: string }>();
  const removal = useApiAction();

  const fail = (name: string, local: LocalError) =>
    setFailed({ name, error: errorMessage({ ok: false, local }) });

  const choose = (file: File) => {
    if (!file.type.startsWith("image/")) {
      fail(file.name, "notImage");
      return;
    }
    startReading(async () => {
      try {
        // Lê a orientação do EXIF: a foto do celular não chega deitada.
        const bitmap = await createImageBitmap(file, {
          imageOrientation: "from-image",
        });
        setFailed(undefined);
        setImage(bitmap);
      } catch {
        fail(file.name, "unreadable");
      }
    });
  };

  const close = () => {
    image?.close();
    setImage(undefined);
  };

  return (
    // O alvo do foco depois de salvar ou remover (o botão que agiu some).
    <div id={id} tabIndex={-1}>
      <ImageInput
        kind="avatar"
        url={url}
        sending={failed?.name}
        pending={reading}
        error={failed?.error ?? removal.error}
        removing={removal.pending}
        onFile={choose}
        onRemove={() =>
          removal.run(
            () => removeImage({ kind: "avatar" }),
            () => {
              router.refresh();
              focusIfLost(id);
            },
          )
        }
        onDismiss={() => setFailed(undefined)}
      />
      {image && (
        <AvatarEditor
          image={image}
          maxBytes={maxBytes}
          onClose={close}
          onSaved={() => {
            close();
            router.refresh();
            focusIfLost(id);
          }}
        />
      )}
    </div>
  );
}
