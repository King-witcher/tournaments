import {
  ICON_PATH,
  LOGO_HEIGHT,
  LOGO_WIDTH,
  WORDMARK_PATH,
} from "@/shared/lib/logo";

type Props = {
  width: number;
  height: number;
  /** "Convite para entrar em X" ou "Inscrição em Y". */
  title: string;
  /** O nome da comunidade, embaixo do título. */
  community: string;
  /** O logo da comunidade (URL ou `data:`). */
  logo?: string | null;
  /** A capa do torneio do link de inscrição (URL ou `data:`). */
  cover?: string | null;
};

/**
 * A prévia de um link de convite (`invite/[token]/opengraph-image.tsx`), para
 * o `ImageResponse`, no fundo escuro do `BrandCard`: o logo da comunidade, o
 * título e o nome dela à esquerda; a capa do torneio à direita, no link de
 * inscrição que tem uma; o logo do Telefrag pequeno embaixo. Só estilo inline
 * e cores soltas, como no `BrandCard`: é o que o `ImageResponse` entende.
 */
export function InviteCard({
  width,
  height,
  title,
  community,
  logo,
  cover,
}: Props) {
  const coverWidth = Math.round(width * 0.42);
  const markWidth = 260;
  return (
    <div
      style={{
        width,
        height,
        display: "flex",
        alignItems: "center",
        gap: 64,
        padding: "0 80px",
        background: "#161616",
        fontFamily: "IBM Plex Sans",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", flex: 1 }}>
        {logo && (
          // biome-ignore lint/performance/noImgElement: o ImageResponse só desenha <img>.
          <img
            src={logo}
            alt=""
            width={144}
            height={144}
            style={{ marginBottom: 40 }}
          />
        )}
        <div style={{ fontSize: 56, fontWeight: 600, color: "#f4f4f4" }}>
          {title}
        </div>
        <div style={{ marginTop: 16, fontSize: 32, color: "#c6c6c6" }}>
          {community}
        </div>
        <svg
          viewBox={`0 0 ${LOGO_WIDTH} ${LOGO_HEIGHT}`}
          width={markWidth}
          height={Math.round((markWidth * LOGO_HEIGHT) / LOGO_WIDTH)}
          fill="#ffffff"
          aria-hidden="true"
          style={{ marginTop: 56 }}
        >
          <path d={ICON_PATH} />
          <path d={WORDMARK_PATH} />
        </svg>
      </div>
      {cover && (
        // biome-ignore lint/performance/noImgElement: o ImageResponse só desenha <img>.
        <img
          src={cover}
          alt=""
          width={coverWidth}
          height={Math.round(coverWidth / 2)}
          style={{ objectFit: "cover" }}
        />
      )}
    </div>
  );
}
