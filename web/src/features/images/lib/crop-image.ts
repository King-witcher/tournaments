/**
 * O recorte quadrado de uma imagem, como o editor da foto de perfil o
 * guarda: o zoom (1 = a imagem cobre o quadrado só no lado menor), o giro
 * em quartos de volta e o deslocamento do centro da imagem em relação ao
 * centro do quadrado, em frações do lado dele (o mesmo recorte vale para a
 * prévia, de qualquer tamanho, e para o arquivo final).
 */
export type Crop = {
  zoom: number;
  /** Quartos de volta no sentido horário, de 0 a 3. */
  turns: number;
  x: number;
  y: number;
};

export const MIN_ZOOM = 1;
export const MAX_ZOOM = 4;

export const INITIAL_CROP: Crop = { zoom: MIN_ZOOM, turns: 0, x: 0, y: 0 };

type Size = { width: number; height: number };

/** Largura e altura da imagem depois do giro. */
function turned(image: Size, turns: number): Size {
  return turns % 2 === 0 ? image : { width: image.height, height: image.width };
}

/** A imagem girada, em lados do quadrado, no zoom do recorte. */
function scaled(image: Size, crop: Crop): Size {
  const { width, height } = turned(image, crop.turns);
  const side = Math.min(width, height);
  return {
    width: (width / side) * crop.zoom,
    height: (height / side) * crop.zoom,
  };
}

/**
 * O recorte com o zoom dentro da faixa e o deslocamento que mantém o
 * quadrado coberto pela imagem (sem faixa vazia na foto).
 */
export function clampCrop(image: Size, crop: Crop): Crop {
  const zoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, crop.zoom));
  const turns = ((crop.turns % 4) + 4) % 4;
  const size = scaled(image, { ...crop, zoom, turns });
  const limitX = (size.width - 1) / 2;
  const limitY = (size.height - 1) / 2;
  return {
    zoom,
    turns,
    x: Math.min(limitX, Math.max(-limitX, crop.x)),
    y: Math.min(limitY, Math.max(-limitY, crop.y)),
  };
}

/** Novo zoom, com o mesmo ponto da imagem no centro do quadrado. */
export function zoomTo(image: Size, crop: Crop, zoom: number): Crop {
  const ratio = zoom / crop.zoom;
  return clampCrop(image, {
    ...crop,
    zoom,
    x: crop.x * ratio,
    y: crop.y * ratio,
  });
}

/** Um quarto de volta (`1` horário, `-1` anti-horário), girando junto o deslocamento. */
export function turn(image: Size, crop: Crop, direction: 1 | -1): Crop {
  const x = direction === 1 ? -crop.y : crop.y;
  const y = direction === 1 ? crop.x : -crop.x;
  return clampCrop(image, { ...crop, turns: crop.turns + direction, x, y });
}

/**
 * Desenha o recorte num quadrado de `side` pixels do contexto (a prévia ou
 * o arquivo final), a partir do canto de cima.
 */
export function drawCrop(
  context: CanvasRenderingContext2D,
  image: ImageBitmap,
  crop: Crop,
  side: number,
) {
  const { width, height } = turned(image, crop.turns);
  const scale = (side / Math.min(width, height)) * crop.zoom;
  context.save();
  context.translate(side / 2 + crop.x * side, side / 2 + crop.y * side);
  context.rotate((crop.turns * Math.PI) / 2);
  context.drawImage(
    image,
    (-image.width * scale) / 2,
    (-image.height * scale) / 2,
    image.width * scale,
    image.height * scale,
  );
  context.restore();
}

/**
 * O recorte num canvas de `side` × `side`, sobre branco (o JPEG não tem
 * transparência). `null` se o navegador não deu o contexto.
 */
export function renderCrop(
  image: ImageBitmap,
  crop: Crop,
  side: number,
): HTMLCanvasElement | null {
  const canvas = document.createElement("canvas");
  canvas.width = side;
  canvas.height = side;
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, side, side);
  context.imageSmoothingQuality = "high";
  drawCrop(context, image, crop, side);
  return canvas;
}
