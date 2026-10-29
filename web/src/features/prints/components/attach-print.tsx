"use client";

import { Attachment } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import {
  type ClipboardEvent,
  useId,
  useRef,
  useState,
  useTransition,
} from "react";

import { useRouter } from "@/i18n/navigation";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import type { PrintTarget } from "../lib/print-target";
import { uploadPrint } from "../lib/upload-print";
import styles from "./attach-print.module.scss";

type Props = {
  target: PrintTarget;
  maxBytes: number;
  /** O torneio exige print e quem vê vai enviar este mapa. */
  required: boolean;
  focusId: string;
};

/**
 * "Anexar print": escolhe um arquivo (no celular, também a câmera ou a
 * galeria) ou cola a imagem com o botão selecionado (Ctrl+V). A imagem vira
 * WebP no navegador, vai direto ao bucket e é registrada na API; o botão fica
 * `pending` o tempo todo, e o erro sai traduzido embaixo dele.
 */
export function AttachPrint({ target, maxBytes, required, focusId }: Props) {
  const t = useTranslations("Prints");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const input = useRef<HTMLInputElement>(null);
  const hintId = useId();
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  const send = (file: File | undefined) => {
    if (!file || pending) return;
    startTransition(async () => {
      const result = await uploadPrint(target, file, maxBytes);
      if (result.ok) {
        setError(undefined);
        router.refresh();
        // Com o mapa cheio, o botão some: o foco vai para a linha do mapa.
        focusIfLost(focusId);
      } else if ("local" in result) {
        setError(t(`errors.${result.local}`));
      } else {
        setError(errorMessage(result.code, result.params));
      }
    });
  };

  // Colar com o botão em foco: o evento sobe do botão até aqui.
  const paste = (event: ClipboardEvent<HTMLDivElement>) => {
    const image = Array.from(event.clipboardData.files).find((f) =>
      f.type.startsWith("image/"),
    );
    if (!image) return;
    event.preventDefault();
    send(image);
  };

  return (
    <div className={styles.attach} onPaste={paste}>
      <ActionWithNote error={error}>
        <Button
          kind="tertiary"
          size="sm"
          renderIcon={Attachment}
          pending={pending}
          aria-label={t("attachTo", { number: target.number })}
          aria-describedby={hintId}
          onClick={() => input.current?.click()}
        >
          {t("attach")}
        </Button>
      </ActionWithNote>
      <input
        ref={input}
        type="file"
        accept="image/*"
        hidden
        onChange={(event) => {
          send(event.target.files?.[0]);
          // Escolher o mesmo arquivo de novo (depois de um erro) dispara outra vez.
          event.target.value = "";
        }}
      />
      <p id={hintId} className={styles.hint}>
        {required ? t("requiredHint") : t("hint")}
      </p>
    </div>
  );
}
