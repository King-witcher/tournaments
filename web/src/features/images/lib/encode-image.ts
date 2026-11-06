/**
 * O logo e a foto de perfil são quadrados e a capa, 2:1, nestes tamanhos no
 * máximo (a foto sai sempre neste, do editor: `crop-image.ts`).
 */
export const SHAPES = {
  logo: { width: 256, height: 256 },
  cover: { width: 1200, height: 600 },
  avatar: { width: 256, height: 256 },
} as const;

export type ImageKind = keyof typeof SHAPES;

/** Qualidades tentadas, da melhor para a menor, até caber no limite. */
const QUALITIES = [0.85, 0.7, 0.55];

export type EncodedImage = {
  blob: Blob;
  type: "image/jpeg";
};

/** Por que a imagem não saiu (o texto fica em `Images.errors`). */
export type EncodeError = "unreadable" | "tooLarge";

function toBlob(canvas: HTMLCanvasElement, type: string, quality: number) {
  return new Promise<Blob | null>((resolve) =>
    canvas.toBlob(resolve, type, quality),
  );
}

/**
 * Corta a imagem pelo centro na proporção de `kind` e a reduz ao tamanho
 * dele (sem aumentar a menor), em JPEG, baixando a qualidade até caber em
 * `maxBytes`. JPEG e não WebP: a prévia dos links (`ImageResponse`) não
 * desenha WebP. O logo sai sem transparência, sobre branco: transparente, um
 * logo escuro sumiria no tema escuro.
 */
export async function encodeImage(
  file: Blob,
  kind: ImageKind,
  maxBytes: number,
): Promise<EncodedImage | EncodeError> {
  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    return "unreadable";
  }
  const target = SHAPES[kind];
  const ratio = target.width / target.height;
  // O maior retângulo da proporção que cabe na imagem, no centro dela.
  const cropWidth = Math.min(bitmap.width, bitmap.height * ratio);
  const cropHeight = cropWidth / ratio;
  const scale = Math.min(1, target.width / cropWidth);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(cropWidth * scale));
  canvas.height = Math.max(1, Math.round(cropHeight * scale));
  const context = canvas.getContext("2d");
  if (!context) return "unreadable";
  // O JPEG não tem transparência: o transparente vira branco (no logo, de
  // propósito).
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.drawImage(
    bitmap,
    (bitmap.width - cropWidth) / 2,
    (bitmap.height - cropHeight) / 2,
    cropWidth,
    cropHeight,
    0,
    0,
    canvas.width,
    canvas.height,
  );
  bitmap.close();
  return encodeCanvas(canvas, maxBytes);
}

/**
 * O canvas já desenhado em JPEG, baixando a qualidade até caber em
 * `maxBytes`.
 */
export async function encodeCanvas(
  canvas: HTMLCanvasElement,
  maxBytes: number,
): Promise<EncodedImage | EncodeError> {
  for (const quality of QUALITIES) {
    const blob = await toBlob(canvas, "image/jpeg", quality);
    if (!blob) return "unreadable";
    if (blob.size <= maxBytes) return { blob, type: "image/jpeg" };
  }
  return "tooLarge";
}
