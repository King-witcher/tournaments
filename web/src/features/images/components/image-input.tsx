"use client";

import { AspectRatio } from "@carbon/react";
import Image from "next/image";
import { useTranslations } from "next-intl";
import { type ReactNode, useRef } from "react";

import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { FileDropZone } from "@/shared/ui/file-drop-zone";
import type { ImageKind } from "../lib/encode-image";
import styles from "./image-input.module.scss";

const ACCEPT = ["image/png", "image/jpeg", "image/webp"];

type Props = {
  kind: ImageKind;
  /** A imagem de agora (do CDN ou a prévia local); sem ela, a área de soltar. */
  url: string | null | undefined;
  /** O título e a dica do campo, quando ele não está numa seção que já os tem. */
  label?: ReactNode;
  hint?: ReactNode;
  /** O nome do arquivo escolhido que está em andamento ou falhou. */
  sending: string | undefined;
  /** Preparando ou mandando o arquivo escolhido. */
  pending: boolean;
  error: string | undefined;
  /** Tirando a imagem (quando tirar chama a API). */
  removing?: boolean;
  onFile: (file: File) => void;
  onRemove: () => void;
  /** O "x" do arquivo que falhou, na área de soltar. */
  onDismiss: () => void;
};

/**
 * A parte visível do logo ou da capa: sem imagem, a área de soltar do Carbon
 * com um arquivo só; com ela, a prévia já cortada como aparece no site,
 * "Trocar" e "Remover", com o erro embaixo. Quem usa decide o que escolher e
 * tirar fazem (`ImageField` salva na hora, `ImagePicker` guarda para depois).
 */
export function ImageInput({
  kind,
  url,
  label,
  hint,
  sending,
  pending,
  error,
  removing = false,
  onFile,
  onRemove,
  onDismiss,
}: Props) {
  const t = useTranslations("Images");
  const input = useRef<HTMLInputElement>(null);

  if (!url) {
    return (
      <FileDropZone
        labelTitle={label}
        labelDescription={hint}
        labelText={t("drop")}
        accept={ACCEPT}
        multiple={false}
        disabled={pending}
        onAddFiles={(_event, { addedFiles }) => {
          const file = addedFiles[0];
          if (file && !pending) onFile(file);
        }}
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
        onDelete={onDismiss}
      />
    );
  }

  return (
    <div className={styles.field}>
      {(label || hint) && (
        <div>
          {label && <p className={styles.title}>{label}</p>}
          {hint && <p className={styles.description}>{hint}</p>}
        </div>
      )}
      <AspectRatio
        ratio={kind === "logo" ? "1x1" : "2x1"}
        className={kind === "logo" ? styles.logo : styles.cover}
      >
        <Image src={url} alt="" fill unoptimized className={styles.image} />
      </AspectRatio>
      <ActionWithNote error={error}>
        <div className={styles.buttons}>
          <Button
            kind="tertiary"
            size="sm"
            pending={pending}
            disabled={removing}
            onClick={() => input.current?.click()}
          >
            {t("change")}
          </Button>
          <Button
            kind="danger--ghost"
            size="sm"
            pending={removing}
            disabled={pending}
            onClick={onRemove}
          >
            {t("remove")}
          </Button>
        </div>
      </ActionWithNote>
      <input
        ref={input}
        type="file"
        accept={ACCEPT.join(",")}
        hidden
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file && !pending) onFile(file);
          // Escolher o mesmo arquivo de novo (depois de um erro) dispara outra vez.
          event.target.value = "";
        }}
      />
    </div>
  );
}
