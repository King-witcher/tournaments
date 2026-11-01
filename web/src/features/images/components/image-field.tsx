"use client";

import { AspectRatio } from "@carbon/react";
import Image from "next/image";
import { useTranslations } from "next-intl";
import { useRef, useState, useTransition } from "react";

import { useRouter } from "@/i18n/navigation";
import { useApiAction } from "@/shared/api/use-api-action";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { FileDropZone } from "@/shared/ui/file-drop-zone";
import {
  type ImageTarget,
  removeImage,
  uploadImage,
} from "../lib/upload-image";
import styles from "./image-field.module.scss";

const ACCEPT = ["image/png", "image/jpeg", "image/webp"];

type Props = {
  target: ImageTarget;
  /** A imagem de agora; nula sem imagem. */
  url: string | null | undefined;
  maxBytes: number;
};

/**
 * O logo ou a capa, que se salvam sozinhos (fora do "Salvar" do formulário):
 * sem imagem, a área de soltar do Carbon com um arquivo só; com ela, a
 * prévia já cortada como aparece no site, "Trocar" e "Remover". A imagem é
 * cortada e convertida no navegador, vai direto ao bucket e é registrada na
 * API; o erro sai traduzido junto do controle.
 */
export function ImageField({ target, url, maxBytes }: Props) {
  const t = useTranslations("Images");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const input = useRef<HTMLInputElement>(null);
  const [pending, startTransition] = useTransition();
  const [sending, setSending] = useState<string>();
  const [error, setError] = useState<string>();
  const removal = useApiAction();

  const send = (file: File | undefined) => {
    if (!file || pending) return;
    setSending(file.name);
    startTransition(async () => {
      const result = await uploadImage(target, file, maxBytes);
      if (result.ok) {
        setError(undefined);
        setSending(undefined);
        router.refresh();
      } else if ("local" in result) {
        setError(t(`errors.${result.local}`));
      } else {
        setError(errorMessage(result.code, result.params));
      }
    });
  };

  const picker = (
    <input
      ref={input}
      type="file"
      accept={ACCEPT.join(",")}
      hidden
      onChange={(event) => {
        send(event.target.files?.[0]);
        // Escolher o mesmo arquivo de novo (depois de um erro) dispara outra vez.
        event.target.value = "";
      }}
    />
  );

  if (!url) {
    return (
      <FileDropZone
        labelText={t("drop")}
        accept={ACCEPT}
        multiple={false}
        disabled={pending}
        onAddFiles={(_event, { addedFiles }) => send(addedFiles[0])}
        files={
          sending
            ? [
                {
                  id: "image",
                  name: sending,
                  status: pending ? "uploading" : "edit",
                  invalid: Boolean(error),
                  errorSubject: error,
                },
              ]
            : []
        }
        onDelete={() => {
          setSending(undefined);
          setError(undefined);
        }}
      />
    );
  }

  return (
    <div className={styles.field}>
      <AspectRatio
        ratio={target.kind === "logo" ? "1x1" : "2x1"}
        className={target.kind === "logo" ? styles.logo : styles.cover}
      >
        <Image src={url} alt="" fill unoptimized className={styles.image} />
      </AspectRatio>
      <ActionWithNote error={error ?? removal.error}>
        <div className={styles.buttons}>
          <Button
            kind="tertiary"
            size="sm"
            pending={pending}
            disabled={removal.pending}
            onClick={() => input.current?.click()}
          >
            {t("change")}
          </Button>
          <Button
            kind="danger--ghost"
            size="sm"
            pending={removal.pending}
            disabled={pending}
            onClick={() =>
              removal.run(
                () => removeImage(target),
                () => router.refresh(),
              )
            }
          >
            {t("remove")}
          </Button>
        </div>
      </ActionWithNote>
      {picker}
    </div>
  );
}
