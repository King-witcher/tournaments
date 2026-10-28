import Image from "next/image";

/**
 * Avatar redondo de 28px; sem imagem, a inicial do nome no mesmo tamanho
 * (ajuda a achar alguém na lista quando ninguém tem avatar).
 */
export function Avatar({ src, name }: { src?: string | null; name?: string }) {
  if (!src) {
    return (
      <span
        aria-hidden
        className="flex size-7 shrink-0 items-center justify-center rounded-full bg-black/10 text-xs font-medium uppercase dark:bg-white/10"
      >
        {name?.trim().charAt(0)}
      </span>
    );
  }
  return (
    <Image src={src} alt="" width={28} height={28} className="rounded-full" />
  );
}
