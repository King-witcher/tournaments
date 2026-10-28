/**
 * O desenho do ícone do site para o `ImageResponse` (`app/icon.tsx`,
 * `app/apple-icon.tsx`): "QC" em branco no azul do Carbon (blue 60), sem
 * arquivo. O `ImageResponse` só entende estilo inline, por isso o `style` e
 * as cores soltas aqui (e só aqui). As letras ficam no meio, longe das bordas
 * que a tela inicial corta.
 */
export function BrandMark({ size }: { size: number }) {
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "#0f62fe",
        color: "#ffffff",
        fontSize: Math.round(size * 0.42),
      }}
    >
      QC
    </div>
  );
}
