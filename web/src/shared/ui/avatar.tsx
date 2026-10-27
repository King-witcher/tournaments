import Image from "next/image";

/** Avatar redondo de 28px; sem imagem, um círculo vazio do mesmo tamanho. */
export function Avatar({ src }: { src?: string | null }) {
  if (!src) {
    return (
      <span className="size-7 rounded-full bg-black/10 dark:bg-white/10" />
    );
  }
  return (
    <Image src={src} alt="" width={28} height={28} className="rounded-full" />
  );
}
