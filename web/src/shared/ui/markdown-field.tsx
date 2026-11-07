"use client";

import { Image } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import {
  type ComponentProps,
  useEffect,
  useId,
  useRef,
  useState,
  useTransition,
} from "react";

import { insertAtCursor } from "@/shared/lib/insert-at-cursor";
import { ActionWithNote } from "./action-with-note";
import { Button } from "./button";
import { FormGroup } from "./carbon/form";
import { Stack } from "./carbon/stack";
import { Tab, TabPanel, TabPanels, Tabs } from "./carbon/tabs";
import styles from "./markdown-field.module.scss";
import { MarkdownText } from "./markdown-text";
import { RequiredLabel } from "./required-label";
import { TabList } from "./tab-list";
import { TextArea } from "./text-area";

/** O envio de uma imagem: a URL que entra no texto, ou o erro já traduzido. */
export type MarkdownImageUpload = (
  file: File,
) => Promise<{ ok: true; url: string } | { ok: false; error: string }>;

type Props = Omit<
  ComponentProps<typeof TextArea>,
  "value" | "defaultValue" | "helperText" | "hideLabel" | "ref"
> & {
  /** Controlado: a pré-via mostra o valor. */
  value: string;
  /** As imagens que a pré-via desenha (`MarkdownText`); sem ele, nenhuma. */
  imagePrefix?: string | null;
  /**
   * Manda a imagem escolhida em "Inserir imagem" (o botão só aparece com
   * ela e fora do `readOnly`).
   */
  onImage?: MarkdownImageUpload;
};

/** O nome do arquivo, sem extensão nem o que quebraria o Markdown, como descrição da imagem. */
function altOf(file: File, fallback: string) {
  const name = file.name
    .replace(/\.[^.]*$/, "")
    .replace(/[[\]()\\]/g, "")
    .replace(/[-_]+/g, " ")
    .trim();
  return name || fallback;
}

/**
 * Campo de texto longo em Markdown (as regras da organização): o rótulo em
 * cima e as abas "Escrever" (o `TextArea`, com a dica da sintaxe embaixo) e
 * "Visualizar" (o texto como vai aparecer, pelo `MarkdownText`, com os links
 * em outra aba para não perder o que foi escrito). As props são as do
 * `TextArea`, menos `helperText` (é a dica) e com `value` obrigatório. A
 * pré-via só renderiza com a aba aberta; com erro (`invalid`), volta para
 * "Escrever", onde o erro aparece.
 *
 * Com `onImage`, "Inserir imagem" em cima do campo: escolhida a imagem, o
 * botão fica pendente, e a URL devolvida entra no cursor como
 * `![nome do arquivo](url)`, numa linha só dela, com a descrição selecionada
 * para ser trocada na hora (sem um diálogo a mais; o nome do arquivo já serve
 * de descrição se a pessoa não mexer). O erro sai embaixo do botão.
 *
 * ```tsx
 * <MarkdownField
 *   labelText={t("rules")}
 *   rows={16}
 *   maxLength={20000}
 *   value={text}
 *   onChange={(e) => setText(e.target.value)}
 *   imagePrefix={settings.rules_image_prefix}
 *   onImage={upload}
 * />
 * ```
 */
export function MarkdownField({
  labelText,
  required,
  value,
  imagePrefix,
  onImage,
  lang,
  invalid,
  readOnly,
  ...props
}: Props) {
  const t = useTranslations("Common.markdown");
  const legendId = useId();
  const area = useRef<HTMLTextAreaElement>(null);
  const file = useRef<HTMLInputElement>(null);
  const [tab, setTab] = useState(0);
  const [sending, startSending] = useTransition();
  const [error, setError] = useState<string>();

  useEffect(() => {
    if (invalid) setTab(0);
  }, [invalid]);

  const send = (chosen: File) => {
    startSending(async () => {
      const result = await onImage?.(chosen);
      if (!result) return;
      if (!result.ok) {
        setError(result.error);
        return;
      }
      setError(undefined);
      // Quem trocou para "Visualizar" enquanto enviava volta a escrever; a
      // inserção espera o campo aparecer, para receber o foco.
      setTab(0);
      const alt = altOf(chosen, t("imageAlt"));
      requestAnimationFrame(() => {
        if (area.current) {
          insertAtCursor(
            area.current,
            `![${alt}](${result.url})`,
            2,
            alt.length,
          );
        }
      });
    });
  };

  return (
    <FormGroup
      legendId={legendId}
      legendText={
        <RequiredLabel required={required}>{labelText}</RequiredLabel>
      }
    >
      <Tabs
        selectedIndex={tab}
        onChange={({ selectedIndex }) => setTab(selectedIndex)}
      >
        <TabList aria-labelledby={legendId}>
          <Tab>{t("write")}</Tab>
          <Tab>{t("preview")}</Tab>
        </TabList>
        <TabPanels>
          <TabPanel className={styles.panel}>
            <Stack gap={4}>
              {onImage && !readOnly && (
                <ActionWithNote error={error}>
                  <Button
                    kind="ghost"
                    size="sm"
                    renderIcon={Image}
                    pending={sending}
                    onClick={() => file.current?.click()}
                  >
                    {t("insertImage")}
                  </Button>
                  <input
                    ref={file}
                    type="file"
                    accept="image/*"
                    hidden
                    onChange={(event) => {
                      const chosen = event.target.files?.[0];
                      // Limpa para a mesma imagem poder ser escolhida de novo.
                      event.target.value = "";
                      if (chosen) send(chosen);
                    }}
                  />
                </ActionWithNote>
              )}
              <TextArea
                {...props}
                ref={area}
                labelText={labelText}
                hideLabel
                required={required}
                readOnly={readOnly}
                lang={lang}
                value={value}
                invalid={invalid}
                helperText={t("hint")}
              />
            </Stack>
          </TabPanel>
          <TabPanel className={styles.panel}>
            {tab === 1 &&
              (value.trim() ? (
                <MarkdownText lang={lang} imagePrefix={imagePrefix} newTab>
                  {value}
                </MarkdownText>
              ) : (
                <p className={styles.empty}>{t("empty")}</p>
              ))}
          </TabPanel>
        </TabPanels>
      </Tabs>
    </FormGroup>
  );
}
