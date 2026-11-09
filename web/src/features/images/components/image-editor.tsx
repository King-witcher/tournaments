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
  initialCrop,
  MAX_ZOOM,
  minZoom,
  renderCrop,
  SHAPES,
  turn,
  zoomTo,
} from "../lib/crop-image";
import { encodeCanvas } from "../lib/encode-image";
import { type FixedImageTarget, sendImage } from "../lib/upload-image";
import { ImageCropper } from "./image-cropper";
import styles from "./image-editor.module.scss";

type Props = {
  target: FixedImageTarget;
  /** A imagem escolhida, já lida (quem abre a fecha depois). */
  image: ImageBitmap;
  maxBytes: number;
  /** A imagem nova foi registrada. */
  onSaved: () => void;
  /** Fechou sem salvar: nada foi enviado. */
  onClose: () => void;
};

/**
 * O Modal que ajusta o logo, a capa ou a foto antes do envio: a área de
 * recorte (arrastar, pinçar, roda do mouse ou teclado), o zoom (`Slider`) e o
 * giro em quartos de volta. Nada sai do navegador antes de salvar; ao salvar,
 * o recorte vira um JPEG no tamanho final (`SHAPES`) e vai ao bucket
 * (`sendImage`): a imagem de antes só é trocada quando a API confere o arquivo.
 * O erro fica no Modal, com o recorte, para tentar de novo.
 */
export function ImageEditor({
  target,
  image,
  maxBytes,
  onSaved,
  onClose,
}: Props) {
  const t = useTranslations("Images");
  const errorMessage = useImageErrorMessage();
  const { kind } = target;
  const shape = SHAPES[kind];
  const [crop, setCrop] = useState<Crop>(() => initialCrop(image, shape));
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  const save = () =>
    startTransition(async () => {
      const canvas = renderCrop(image, shape, crop);
      const encoded = canvas
        ? await encodeCanvas(canvas, maxBytes)
        : "unreadable";
      const result =
        typeof encoded === "string"
          ? ({ ok: false, local: encoded } as const)
          : await sendImage(target, encoded);
      if (result.ok) {
        onSaved();
      } else {
        setError(errorMessage(result));
      }
    });

  return (
    <FormModal
      title={t(`${kind}.editTitle`)}
      submitLabel={t(`${kind}.save`)}
      onSubmit={save}
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
