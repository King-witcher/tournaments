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
  clampCrop,
  INITIAL_CROP,
  MAX_ZOOM,
  MIN_ZOOM,
  renderCrop,
  turn,
  zoomTo,
} from "../lib/crop-image";
import { encodeCanvas, SHAPES } from "../lib/encode-image";
import { sendImage } from "../lib/upload-image";
import { AvatarCropper } from "./avatar-cropper";
import styles from "./avatar-editor.module.scss";

type Props = {
  /** A imagem escolhida, já lida (quem abre a fecha depois). */
  image: ImageBitmap;
  maxBytes: number;
  /** A foto nova foi registrada. */
  onSaved: () => void;
  /** Fechou sem salvar: nada foi enviado. */
  onClose: () => void;
};

/**
 * O Modal que ajusta a foto de perfil antes do envio: a área de recorte
 * (arrastar, pinçar, roda do mouse ou teclado), o zoom (`Slider`) e o giro
 * em quartos de volta. Nada sai do navegador antes de "Salvar foto"; ao
 * salvar, o recorte vira um JPEG quadrado e segue o caminho do logo
 * (`sendImage`): a foto de antes só é trocada quando a API confere o arquivo
 * no bucket. O erro fica no Modal, com o recorte, para tentar de novo.
 */
export function AvatarEditor({ image, maxBytes, onSaved, onClose }: Props) {
  const t = useTranslations("Images.avatar");
  const errorMessage = useImageErrorMessage();
  const [crop, setCrop] = useState<Crop>(() => clampCrop(image, INITIAL_CROP));
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  const save = () =>
    startTransition(async () => {
      const canvas = renderCrop(image, crop, SHAPES.avatar.width);
      const encoded = canvas
        ? await encodeCanvas(canvas, maxBytes)
        : "unreadable";
      const result =
        typeof encoded === "string"
          ? ({ ok: false, local: encoded } as const)
          : await sendImage({ kind: "avatar" }, encoded);
      if (result.ok) {
        onSaved();
      } else {
        setError(errorMessage(result));
      }
    });

  return (
    <FormModal
      title={t("editTitle")}
      submitLabel={t("save")}
      onSubmit={save}
      onClose={onClose}
      pending={pending}
      error={error}
    >
      <AvatarCropper
        image={image}
        crop={crop}
        onChange={setCrop}
        label={t("area")}
      />
      <Stack gap={3}>
        <Slider
          labelText={t("zoom")}
          min={MIN_ZOOM * 100}
          max={MAX_ZOOM * 100}
          step={1}
          value={Math.round(crop.zoom * 100)}
          formatLabel={(value) => `${value}%`}
          hideTextInput
          onChange={({ value }) =>
            setCrop((c) => zoomTo(image, c, value / 100))
          }
        />
        <div className={styles.turns}>
          <IconButton
            label={t("rotateLeft")}
            onClick={() => setCrop((c) => turn(image, c, -1))}
          >
            <RotateCounterclockwise />
          </IconButton>
          <IconButton
            label={t("rotateRight")}
            onClick={() => setCrop((c) => turn(image, c, 1))}
          >
            <RotateClockwise />
          </IconButton>
        </div>
      </Stack>
    </FormModal>
  );
}
