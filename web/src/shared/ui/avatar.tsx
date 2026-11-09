import Image from "next/image";

import { cn } from "@/shared/lib/cn";
import styles from "./avatar.module.scss";

/** Lado em px de cada tamanho, para o `next/image` pedir a imagem certa. */
const PIXELS = { sm: 24, md: 32, lg: 48 } as const;

type Props = {
  src?: string | null;
  /** Sem imagem, a inicial do nome (ajuda a achar alguém na lista). */
  name?: string;
  /** `sm` 24px (linha de tabela, cabeçalho), `md` 32px, `lg` 48px (placar). */
  size?: keyof typeof PIXELS;
  /**
   * `circle` (o padrão) para gente; `square` para uma coisa, como o logo de
   * uma comunidade.
   */
  shape?: "circle" | "square";
};

/**
 * Só o avatar do Discord passa pela otimização de imagens da Vercel (que tem
 * cota, e está liberada só para ele no `next.config.ts`): o logo e a foto
 * enviada vêm do CDN do site já no tamanho.
 */
const OPTIMIZED = "https://cdn.discordapp.com/";

/**
 * A inicial: a primeira letra ou número ("[AR] Lobo" dá "A"), senão o
 * primeiro caractere inteiro. `charAt(0)` partia o emoji ao meio ("🔥 Inferno"
 * virava "�" e o servidor e o navegador discordavam na hidratação).
 */
function initial(name: string | undefined) {
  const text = name?.trim() ?? "";
  return text.match(/[\p{L}\p{N}]/u)?.[0] ?? Array.from(text)[0] ?? "";
}

/** Avatar redondo (ou quadrado); decorativo (o nome sempre aparece ao lado). */
export function Avatar({ src, name, size = "sm", shape = "circle" }: Props) {
  const className = cn(styles.avatar, styles[size], styles[shape]);
  if (!src) {
    return (
      <span aria-hidden className={className}>
        {initial(name)}
      </span>
    );
  }
  return (
    <Image
      src={src}
      alt=""
      width={PIXELS[size]}
      height={PIXELS[size]}
      unoptimized={!src.startsWith(OPTIMIZED)}
      className={className}
      draggable={false}
    />
  );
}
