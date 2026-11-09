"use client";

import Image from "next/image";
import { useTranslations } from "next-intl";
import { type ReactNode, useRef } from "react";

import { cn } from "@/shared/lib/cn";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { AspectRatio } from "@/shared/ui/carbon/aspect-ratio";
import { FormGroup } from "@/shared/ui/carbon/form";
import { FileDropZone } from "@/shared/ui/file-drop-zone";
import type { ImageKind } from "../lib/crop-image";
import styles from "./image-input.module.scss";

const ACCEPT = ["image/png", "image/jpeg", "image/webp"];

type Props = {
  kind: ImageKind;
  /** A imagem de agora (do CDN ou a prévia local); sem ela, a área de soltar. */
  url: string | null | undefined;
  /** O rótulo do campo (a `legend` do `FormGroup`, com a forma do de um campo de texto). */
  label?: ReactNode;
  /** A dica embaixo do campo, como o `helperText`. */
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
 * A parte visível do logo, da capa ou da foto de perfil, com o rótulo e a dica
 * de um campo de formulário, para alinhar com os campos ao lado: sem imagem, a
 * área de soltar do Carbon com um arquivo só, na largura da coluna; com ela, a
 * prévia já cortada como aparece no site (a foto redonda), "Trocar" e
 * "Remover", com o erro embaixo (no logo, pequeno, os botões ao lado dele).
 * Quem usa decide o que escolher e tirar fazem (`ImageField`).
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

  const control = (
    <>
      {url ? (
        <div className={cn(styles.current, kind === "logo" && styles.beside)}>
          <AspectRatio
            ratio={kind === "cover" ? "2x1" : "1x1"}
            className={styles[kind]}
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
      ) : (
        <FileDropZone
          className={styles.drop}
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
      )}
      {hint && <p className={styles.hint}>{hint}</p>}
    </>
  );
  return label ? <FormGroup legendText={label}>{control}</FormGroup> : control;
}
