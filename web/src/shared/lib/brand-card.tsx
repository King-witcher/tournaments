import { ICON_PATH, LOGO_HEIGHT, LOGO_WIDTH, WORDMARK_PATH } from "./logo";

type Props = {
  /** Largura e altura da imagem. */
  width: number;
  height: number;
  title: string;
  description: string;
};

/**
 * A prévia dos links do site (`app/[locale]/opengraph-image.tsx`), para o
 * `ImageResponse`: o logo inteiro (ícone e nome) em branco no fundo do tema
 * escuro do Carbon (gray 100), como o `BrandMark`, com o título e a frase
 * embaixo, na fonte do site. Só estilo inline e cores soltas, como no
 * `BrandMark`: é o que o `ImageResponse` entende. Tudo fica à esquerda e no
 * meio da altura, longe das bordas que alguns apps cortam.
 */
export function BrandCard({ width, height, title, description }: Props) {
  const logoWidth = Math.round(width * 0.6);
  return (
    <div
      style={{
        width,
        height,
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        padding: "0 96px",
        background: "#161616",
        fontFamily: "IBM Plex Sans",
      }}
    >
      <svg
        viewBox={`0 0 ${LOGO_WIDTH} ${LOGO_HEIGHT}`}
        width={logoWidth}
        height={Math.round((logoWidth * LOGO_HEIGHT) / LOGO_WIDTH)}
        fill="#ffffff"
        aria-hidden="true"
      >
        <path d={ICON_PATH} />
        <path d={WORDMARK_PATH} />
      </svg>
      <div
        style={{
          marginTop: 56,
          fontSize: 60,
          fontWeight: 600,
          color: "#f4f4f4",
        }}
      >
        {title}
      </div>
      <div
        style={{
          marginTop: 16,
          fontSize: 32,
          color: "#c6c6c6",
        }}
      >
        {description}
      </div>
    </div>
  );
}
