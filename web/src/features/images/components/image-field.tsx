"use client";

import { memo, type ReactNode, useId, useState, useTransition } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import type { ImageDraft } from "../hooks/use-image-draft";
import { useImageErrorMessage } from "../hooks/use-image-error-message";
import type { EncodedImage } from "../lib/encode-image";
import type { LocalError } from "../lib/upload-image";
import { ImageEditor } from "./image-editor";
import { ImageInput } from "./image-input";

type Props = {
  /** A imagem pendente, do formulário que tem o Salvar (`useImageDraft`). */
  draft: ImageDraft;
  /** A imagem salva; nula sem ela. */
  url: string | null | undefined;
  maxBytes: number;
  /** O rótulo e a dica do campo, num formulário. */
  label?: ReactNode;
  hint?: ReactNode;
  /** O formulário está salvando: nada muda até acabar. */
  saving?: boolean;
};

/**
 * O logo, a capa ou a foto como um campo de formulário: escolher o arquivo
 * abre o editor (`ImageEditor`), e "Aplicar" deixa o recorte como prévia;
 * "Remover" deixa o campo vazio. Nada disso sai do navegador antes do Salvar
 * do formulário, que manda a pendência por `draft.apply`, e "Desfazer" volta
 * à imagem salva. O erro do Salvar fica aqui, junto da imagem; o arquivo que
 * não dá para ler fica com o erro na área de soltar. `draft` estável: fica de
 * fora do que recarrega a cada tecla do formulário.
 */
export const ImageField = memo(function ImageField({
  draft,
  url,
  maxBytes,
  label,
  hint,
  saving = false,
}: Props) {
  const errorMessage = useImageErrorMessage();
  const id = useId();
  const [image, setImage] = useState<ImageBitmap>();
  const [reading, startReading] = useTransition();
  const [failed, setFailed] = useState<{ name: string; error: string }>();
  const { kind } = draft.target;

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

  const apply = (encoded: EncodedImage) => {
    draft.replace(encoded);
    close();
    focusIfLost(id);
  };

  // Sem imagem salva, tirar o recorte é só voltar ao campo vazio.
  const remove = () => {
    if (url) draft.remove();
    else draft.discard();
    focusIfLost(id);
  };

  const undo = () => {
    draft.discard();
    focusIfLost(id);
  };

  return (
    // O alvo do foco depois de aplicar, remover ou desfazer (o botão que agiu
    // some).
    <div id={id} tabIndex={-1}>
      <ImageInput
        kind={kind}
        url={draft.removing ? undefined : (draft.preview ?? url)}
        label={label}
        hint={hint}
        sending={failed?.name}
        pending={reading}
        disabled={saving}
        error={failed?.error ?? (draft.failure && errorMessage(draft.failure))}
        onFile={choose}
        onRemove={remove}
        onUndo={draft.dirty ? undo : undefined}
        onDismiss={() => setFailed(undefined)}
      />
      {image && (
        <ImageEditor
          kind={kind}
          image={image}
          maxBytes={maxBytes}
          onApply={apply}
          onClose={close}
        />
      )}
    </div>
  );
});
