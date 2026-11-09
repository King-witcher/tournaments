"use client";

import { RotateClockwise, RotateCounterclockwise } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useState, useTransition } from "react";

import { Stack } from "@/shared/ui/carbon/stack";
import { FormModal } from "@/shared/ui/form-modal";
import { IconButton } from "@/shared/ui/icon-button";
import { Slider } from "@/shared/ui/slider";
import { useImageErrorMessage } from "../hooks/use-image-error-message";
import {
  type Crop,
  type ImageKind,
  initialCrop,
  MAX_ZOOM,
  minZoom,
  renderCrop,
  SHAPES,
  turn,
  zoomTo,
} from "../lib/crop-image";
import { type EncodedImage, encodeCanvas } from "../lib/encode-image";
import { ImageCropper } from "./image-cropper";
import styles from "./image-editor.module.scss";

type Props = {
  kind: ImageKind;
  /** A imagem escolhida, já lida (quem abre a fecha depois). */
  image: ImageBitmap;
  maxBytes: number;
  /** O recorte pronto, no arquivo final: quem abre guarda até o Salvar. */
  onApply: (encoded: EncodedImage) => void;
  /** Fechou sem aplicar. */
  onClose: () => void;
};

/**
 * O Modal que ajusta o logo, a capa ou a foto antes do envio: a área de
 * recorte (arrastar, pinçar, roda do mouse ou teclado), o zoom (`Slider`) e o
 * giro em quartos de volta. "Aplicar" transforma o recorte num JPEG no tamanho
 * final (`SHAPES`) e o devolve a quem abriu, sem enviar nada; quem envia é o
 * Salvar do formulário (`useImageDraft`). O erro da conversão fica no Modal,
 * com o recorte, para tentar de novo.
 */
export function ImageEditor({
  kind,
  image,
  maxBytes,
  onApply,
  onClose,
}: Props) {
  const t = useTranslations("Images");
  const errorMessage = useImageErrorMessage();
  const shape = SHAPES[kind];
  const [crop, setCrop] = useState<Crop>(() => initialCrop(image, shape));
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  const apply = () =>
    startTransition(async () => {
      const canvas = renderCrop(image, shape, crop);
      const encoded = canvas
        ? await encodeCanvas(canvas, maxBytes)
        : "unreadable";
      if (typeof encoded === "string") {
        setError(errorMessage({ ok: false, local: encoded }));
      } else {
        onApply(encoded);
      }
    });

  return (
    <FormModal
      title={t(`${kind}.editTitle`)}
      submitLabel={t("editor.apply")}
      onSubmit={apply}
      onClose={onClose}
      pending={pending}
      error={error}
    >
      <ImageCropper
        kind={kind}
        image={image}
        crop={crop}
        onChange={setCrop}
        label={t(`${kind}.area`)}
      />
      <Stack gap={3}>
        <Slider
          labelText={t("editor.zoom")}
          min={Math.floor(minZoom(image, shape, crop.turns) * 100)}
          max={MAX_ZOOM * 100}
          step={1}
          value={Math.round(crop.zoom * 100)}
          formatLabel={(value) => `${value}%`}
          hideTextInput
          onChange={({ value }) =>
            setCrop((c) => zoomTo(image, shape, c, value / 100))
          }
        />
        <div className={styles.turns}>
          <IconButton
            label={t("editor.rotateLeft")}
            onClick={() => setCrop((c) => turn(image, shape, c, -1))}
          >
            <RotateCounterclockwise />
          </IconButton>
          <IconButton
            label={t("editor.rotateRight")}
            onClick={() => setCrop((c) => turn(image, shape, c, 1))}
          >
            <RotateClockwise />
          </IconButton>
        </div>
      </Stack>
    </FormModal>
  );
}
