"use client";

import { useTranslations } from "next-intl";
import { type ComponentProps, type ReactNode, useId } from "react";

import { cn } from "@/shared/lib/cn";
import {
  FileUploaderDropContainer,
  FileUploaderItem,
} from "./carbon/file-uploader";
import { FormItem } from "./carbon/form";
import styles from "./file-drop-zone.module.scss";
import { RequiredLabel } from "./required-label";

// O rótulo do ícone de cada arquivo, que muda com o estado.
const STATUS = {
  edit: "remove",
  uploading: "uploading",
  complete: "uploaded",
} as const;

/** Um arquivo da lista, no estado que o chamador acompanha. */
export type DropZoneFile = {
  /** Identifica o arquivo; volta no `onDelete`. */
  id: string;
  name: string;
  /** `uploading` mostra o indicador, `complete` o visto e `edit` o "x". */
  status: "uploading" | "edit" | "complete";
  invalid?: boolean;
  /** O erro, curto e já traduzido ("O arquivo passa de 2 MB."). */
  errorSubject?: string;
  /** O detalhe do erro, quando o título não basta. */
  errorBody?: string;
};

type Props = Omit<
  ComponentProps<typeof FileUploaderDropContainer>,
  "id" | "labelText"
> & {
  /** O título visível em cima da área. */
  labelTitle?: ReactNode;
  /** A dica embaixo do título: tipos e tamanho aceitos. */
  labelDescription?: ReactNode;
  /** O texto da área; padrão "Arraste os arquivos para cá ou clique para escolher". */
  labelText?: string;
  /** O `id` do `<input type="file">`; gerado quando não vem. */
  id?: string;
  files: DropZoneFile[];
  /** O "x" de um arquivo em `edit`. */
  onDelete: (id: string) => void;
  /** A altura das linhas dos arquivos. */
  size?: "sm" | "md" | "lg";
};

/**
 * O envio por arrastar do Carbon (`FileUploaderDropContainer`): a área que
 * recebe os arquivos soltos ou abre a escolha ao clicar, com o título, a dica
 * e a lista dos arquivos embaixo (`FileUploaderItem`). Para uma área maior,
 * em que arrastar ajuda (várias capturas de tela); só o botão é o
 * `FileUploader`. A lista é do chamador: `onAddFiles` entrega os novos (com
 * `invalidFileType` nos que o `accept` ou o `maxFileSize` recusam), e cada
 * arquivo tem o próprio estado e erro. `required` põe a estrela no título.
 * Traduz a área e os rótulos dos ícones.
 */
export function FileDropZone({
  labelTitle,
  labelDescription,
  labelText,
  id,
  files,
  onDelete,
  size,
  disabled,
  required,
  ...props
}: Props) {
  const t = useTranslations("Common.fileUploader");
  const generated = useId();
  const inputId = id ?? generated;
  const descriptionId = `${inputId}-description`;
  return (
    <FormItem>
      {labelTitle && (
        <p className={cn(styles.title, disabled && styles.disabled)}>
          <RequiredLabel required={required}>{labelTitle}</RequiredLabel>
        </p>
      )}
      {labelDescription && (
        <p
          id={descriptionId}
          className={cn(styles.description, disabled && styles.disabled)}
        >
          {labelDescription}
        </p>
      )}
      <FileUploaderDropContainer
        id={inputId}
        labelText={labelText ?? t("drop")}
        disabled={disabled}
        required={required}
        aria-describedby={labelDescription ? descriptionId : undefined}
        {...props}
      />
      {files.length > 0 && (
        <div className={styles.files}>
          {files.map((file) => (
            <FileUploaderItem
              key={file.id}
              uuid={file.id}
              name={file.name}
              status={file.status}
              iconDescription={t(STATUS[file.status])}
              invalid={file.invalid}
              errorSubject={file.errorSubject}
              errorBody={file.errorBody}
              size={size}
              disabled={disabled}
              onDelete={(_event, { uuid }) => onDelete(uuid)}
            />
          ))}
        </div>
      )}
    </FormItem>
  );
}
