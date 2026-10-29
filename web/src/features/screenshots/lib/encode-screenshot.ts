/** Largura máxima do screenshot: o placar do jogo continua legível. */
export const MAX_WIDTH = 1920;

/** Qualidades tentadas, da melhor para a menor, até caber no limite. */
const QUALITIES = [0.8, 0.65, 0.5];

export type EncodedScreenshot = {
  blob: Blob;
  type: "image/webp" | "image/jpeg";
};

/** Por que a imagem não saiu (o texto fica em `Screenshots.errors`). */
export type EncodeError = "unreadable" | "tooLarge";

function toBlob(canvas: HTMLCanvasElement, type: string, quality: number) {
  return new Promise<Blob | null>((resolve) =>
    canvas.toBlob(resolve, type, quality),
  );
}

/**
 * Converte a imagem escolhida ou colada para WebP no navegador, com no
 * máximo `MAX_WIDTH` de largura e qualidade ~0,8, baixando a qualidade até
 * caber em `maxBytes`. O Safari não gera WebP no canvas (devolve PNG): nele,
 * sai JPEG.
 */
export async function encodeScreenshot(
  file: Blob,
  maxBytes: number,
): Promise<EncodedScreenshot | EncodeError> {
  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    return "unreadable";
  }
  const scale = Math.min(1, MAX_WIDTH / bitmap.width);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bitmap.width * scale));
  canvas.height = Math.max(1, Math.round(bitmap.height * scale));
  const context = canvas.getContext("2d");
  if (!context) return "unreadable";
  context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  bitmap.close();

  for (const quality of QUALITIES) {
    const webp = await toBlob(canvas, "image/webp", quality);
    const blob =
      webp?.type === "image/webp"
        ? webp
        : await toBlob(canvas, "image/jpeg", quality);
    if (!blob) return "unreadable";
    if (blob.size <= maxBytes) {
      return {
        blob,
        type: blob.type === "image/webp" ? "image/webp" : "image/jpeg",
      };
    }
  }
  return "tooLarge";
}
