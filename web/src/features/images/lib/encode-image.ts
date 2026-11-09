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

/** Largura máxima de uma imagem das regras: o dobro da coluna de leitura, para telas de alta densidade. */
export const RULES_IMAGE_WIDTH = 1600;

/**
 * Reduz a imagem das regras até `RULES_IMAGE_WIDTH` de largura, sem cortar
 * nem aumentar (a proporção é a do diagrama ou da captura), em JPEG, baixando
 * a qualidade até caber em `maxBytes`. O transparente vira branco.
 */
export async function encodeScaled(
  file: Blob,
  maxBytes: number,
): Promise<EncodedImage | EncodeError> {
  let bitmap: ImageBitmap;
  try {
    bitmap = await createImageBitmap(file);
  } catch {
    return "unreadable";
  }
  const scale = Math.min(1, RULES_IMAGE_WIDTH / bitmap.width);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bitmap.width * scale));
  canvas.height = Math.max(1, Math.round(bitmap.height * scale));
  const context = canvas.getContext("2d");
  if (!context) return "unreadable";
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  bitmap.close();
  return encodeCanvas(canvas, maxBytes);
}

/**
 * O canvas já desenhado em JPEG, baixando a qualidade até caber em
 * `maxBytes`. JPEG: a imagem vai como está para a prévia dos links, e todo
 * robô de rede social o lê.
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
