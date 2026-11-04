import { ICON_PATH, LOGO_HEIGHT, LOGO_WIDTH, WORDMARK_PATH } from "./logo";

type Props = {
  /** Largura e altura da imagem. */
  width: number;
  height: number;
};

/**
 * A prévia dos links do site (`app/[locale]/opengraph-image.tsx`), para o
 * `ImageResponse`: só o logo inteiro (ícone e nome), em branco no fundo do tema
 * escuro do Carbon (gray 100), como o `BrandMark`. O texto fica nas tags, que
 * toda rede mostra em volta da imagem. Só estilo inline e cores soltas, como no
 * `BrandMark`: é o que o `ImageResponse` entende.
 */
export function BrandCard({ width, height }: Props) {
  const logoWidth = Math.round(width * 0.68);
  return (
    <div
      style={{
        width,
        height,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "#161616",
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
    </div>
  );
}
