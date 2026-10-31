import { ICON_PATH, ICON_WIDTH, LOGO_HEIGHT } from "./logo";

/**
 * O desenho do ícone do site para o `ImageResponse` (`app/icon.tsx`,
 * `app/apple-icon.tsx`): o ícone do logo em branco no fundo do tema escuro
 * do Carbon (gray 100), como no banner. O `ImageResponse` só entende estilo
 * inline, por isso o `style` e as cores soltas aqui (e só aqui). O desenho
 * fica no meio, com 62% da altura, longe das bordas que a tela inicial
 * corta. O `favicon.ico` é o mesmo desenho, maior no quadro (a aba não
 * corta nada).
 */
export function BrandMark({ size }: { size: number }) {
  const height = Math.round(size * 0.62);
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "#161616",
      }}
    >
      <svg
        viewBox={`0 0 ${ICON_WIDTH} ${LOGO_HEIGHT}`}
        width={Math.round((height * ICON_WIDTH) / LOGO_HEIGHT)}
        height={height}
        fill="#ffffff"
        aria-hidden="true"
      >
        <path d={ICON_PATH} />
      </svg>
    </div>
  );
}
