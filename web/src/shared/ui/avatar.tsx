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
};

/** Avatar redondo; decorativo (o nome sempre aparece ao lado). */
export function Avatar({ src, name, size = "sm" }: Props) {
  const className = cn(styles.avatar, styles[size]);
  if (!src) {
    return (
      <span aria-hidden className={className}>
        {name?.trim().charAt(0)}
      </span>
    );
  }
  return (
    <Image
      src={src}
      alt=""
      width={PIXELS[size]}
      height={PIXELS[size]}
      className={className}
    />
  );
}
