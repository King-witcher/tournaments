/**
 * O tamanho do arquivo de cada imagem. `fit`: o zoom pode ir até a imagem
 * inteira caber, com o resto em branco (um logo horizontal não perde as
 * pontas); sem ele, a imagem sempre preenche o recorte.
 */
export const SHAPES = {
  logo: { width: 256, height: 256, fit: true },
  cover: { width: 1200, height: 600, fit: false },
  avatar: { width: 256, height: 256, fit: false },
} as const;

export type ImageKind = keyof typeof SHAPES;

type CropShape = (typeof SHAPES)[ImageKind];

/**
 * O recorte de uma imagem, como o editor o guarda: o zoom (1 = a imagem
 * preenche o quadro só no lado que falta), o giro em quartos de volta e o
 * deslocamento do centro da imagem em relação ao centro do quadro, em frações
 * da largura dele (o mesmo recorte vale para a prévia, de qualquer tamanho, e
 * para o arquivo final).
 */
export type Crop = {
  zoom: number;
  /** Quartos de volta no sentido horário, de 0 a 3. */
  turns: number;
  x: number;
  y: number;
};

export const MAX_ZOOM = 4;

type Size = { width: number; height: number };

/** A altura do quadro, em larguras dele. */
function frameHeight(shape: CropShape) {
  return shape.height / shape.width;
}

/** Largura e altura da imagem depois do giro. */
function turned(image: Size, turns: number): Size {
  return turns % 2 === 0 ? image : { width: image.height, height: image.width };
}

/** A imagem girada, em larguras do quadro, no zoom do recorte. */
function scaled(image: Size, shape: CropShape, crop: Crop): Size {
  const { width, height } = turned(image, crop.turns);
  const scale = Math.max(1 / width, frameHeight(shape) / height) * crop.zoom;
  return { width: width * scale, height: height * scale };
}

/** O menor zoom: 1 ou, com `fit`, o que faz a imagem inteira caber no quadro. */
export function minZoom(image: Size, shape: CropShape, turns: number) {
  if (!shape.fit) return 1;
  const { width, height } = turned(image, turns);
  const byWidth = 1 / width;
  const byHeight = frameHeight(shape) / height;
  return Math.min(byWidth, byHeight) / Math.max(byWidth, byHeight);
}

/**
 * O recorte com o zoom dentro da faixa e o deslocamento que mantém a imagem
 * sobre o quadro: maior que ele, sem faixa vazia; menor (só com `fit`), sem
 * sair dele.
 */
export function clampCrop(image: Size, shape: CropShape, crop: Crop): Crop {
  const turns = ((crop.turns % 4) + 4) % 4;
  const zoom = Math.min(
    MAX_ZOOM,
    Math.max(minZoom(image, shape, turns), crop.zoom),
  );
  const size = scaled(image, shape, { ...crop, zoom, turns });
  const limitX = Math.abs(size.width - 1) / 2;
  const limitY = Math.abs(size.height - frameHeight(shape)) / 2;
  return {
    zoom,
    turns,
    x: Math.min(limitX, Math.max(-limitX, crop.x)),
    y: Math.min(limitY, Math.max(-limitY, crop.y)),
  };
}

/**
 * O recorte ao abrir o editor, no centro: com `fit`, a imagem inteira; sem
 * ele, preenchendo o quadro.
 */
export function initialCrop(image: Size, shape: CropShape): Crop {
  return { zoom: minZoom(image, shape, 0), turns: 0, x: 0, y: 0 };
}

/** Novo zoom, com o mesmo ponto da imagem no centro do quadro. */
export function zoomTo(
  image: Size,
  shape: CropShape,
  crop: Crop,
  zoom: number,
): Crop {
  const ratio = zoom / crop.zoom;
  return clampCrop(image, shape, {
    ...crop,
    zoom,
    x: crop.x * ratio,
    y: crop.y * ratio,
  });
}

/** Um quarto de volta (`1` horário, `-1` anti-horário), girando junto o deslocamento. */
export function turn(
  image: Size,
  shape: CropShape,
  crop: Crop,
  direction: 1 | -1,
): Crop {
  const x = direction === 1 ? -crop.y : crop.y;
  const y = direction === 1 ? crop.x : -crop.x;
  return clampCrop(image, shape, {
    ...crop,
    turns: crop.turns + direction,
    x,
    y,
  });
}

/**
 * Desenha o recorte num quadro de `width` pixels de largura do contexto (a
 * prévia ou o arquivo final), a partir do canto de cima, sobre branco: o JPEG
 * não tem transparência, e a prévia mostra o que vai sair.
 */
export function drawCrop(
  context: CanvasRenderingContext2D,
  image: ImageBitmap,
  shape: CropShape,
  crop: Crop,
  width: number,
) {
  const height = width * frameHeight(shape);
  const size = turned(image, crop.turns);
  const scale = Math.max(width / size.width, height / size.height) * crop.zoom;
  context.save();
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, width, height);
  context.imageSmoothingQuality = "high";
  context.translate(width / 2 + crop.x * width, height / 2 + crop.y * width);
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
 * O recorte num canvas do tamanho final de `shape`. `null` se o navegador não
 * deu o contexto.
 */
export function renderCrop(
  image: ImageBitmap,
  shape: CropShape,
  crop: Crop,
): HTMLCanvasElement | null {
  const canvas = document.createElement("canvas");
  canvas.width = shape.width;
  canvas.height = shape.height;
  const context = canvas.getContext("2d");
  if (!context) return null;
  drawCrop(context, image, shape, crop, shape.width);
  return canvas;
}
