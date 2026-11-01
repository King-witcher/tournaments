"use client";

import {
  FileUploader as CarbonFileUploader,
  type FileUploaderProps,
} from "@carbon/react";
import { useTranslations } from "next-intl";

// O rótulo do ícone de cada arquivo, que muda com o estado.
const STATUS = {
  edit: "remove",
  uploading: "uploading",
  complete: "uploaded",
} as const;

type Props = Omit<FileUploaderProps, "buttonLabel" | "iconDescription"> & {
  /** O texto do botão; padrão "Adicionar arquivo". */
  buttonLabel?: string;
};

/**
 * `FileUploader` do Carbon: o botão que abre a escolha de arquivos e a lista
 * dos escolhidos embaixo. `labelTitle` é o título e `labelDescription`, a
 * dica (tipos e tamanho aceitos, que o `accept` e o `maxFileSize` cobram);
 * `filenameStatus` vale para todos: `edit` põe o "x" que tira da lista,
 * `uploading` o indicador e `complete` o visto. A lista fica dentro do
 * componente; para ter o estado de cada arquivo (erro, envio por arquivo)
 * ou soltar arquivos arrastados, use o `FileDropZone`. Traduz o botão e os
 * rótulos dos ícones.
 */
export function FileUploader({ buttonLabel, ...props }: Props) {
  const t = useTranslations("Common.fileUploader");
  return (
    <CarbonFileUploader
      buttonLabel={buttonLabel ?? t("add")}
      iconDescription={t(STATUS[props.filenameStatus])}
      {...props}
    />
  );
}
