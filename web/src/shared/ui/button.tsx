import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

// Nomes de variante e tamanho iguais aos do shadcn, para trocar sem mexer em quem usa.
const variants = {
  default:
    "bg-black font-medium text-white hover:bg-black/80 dark:bg-white dark:text-black dark:hover:bg-white/80",
  outline:
    "border border-black/25 hover:bg-black/5 dark:border-white/30 dark:hover:bg-white/10",
  // Vermelho cheio só no "sim" de uma ação que não se desfaz.
  destructive:
    "bg-red-700 font-medium text-white hover:bg-red-800 dark:bg-red-600 dark:hover:bg-red-700",
  // Sublinhado sempre: sem hover (celular) não daria para saber que é clicável.
  link: "underline underline-offset-4 hover:decoration-2",
};

// No celular, alvo de toque de 44px; no desktop, a altura compacta de antes.
const sizes = {
  default: "min-h-11 px-4 py-2 sm:min-h-9",
  sm: "min-h-11 px-3 py-1.5 sm:min-h-8",
};

type ButtonStyle = {
  variant?: keyof typeof variants;
  size?: keyof typeof sizes;
};

/** Classes do botão, para usar também em `<Link>`. `link` ignora o tamanho. */
export function buttonVariants({
  variant = "default",
  size = "default",
}: ButtonStyle = {}) {
  return cn(
    "inline-flex items-center justify-center gap-2 rounded-md text-sm disabled:cursor-not-allowed disabled:opacity-50",
    variants[variant],
    variant === "link" ? "min-h-11 sm:min-h-0" : sizes[size],
  );
}

/**
 * Botão. `pending`: a ação está em andamento; desabilita (sem clique duplo) e
 * mostra um indicador girando ao lado do texto, para quem clicou saber que
 * algo vai acontecer.
 */
export function Button({
  variant,
  size,
  className,
  type = "button",
  pending,
  disabled,
  children,
  ...props
}: ComponentProps<"button"> & ButtonStyle & { pending?: boolean }) {
  return (
    <button
      type={type}
      className={cn(buttonVariants({ variant, size }), className)}
      disabled={disabled || pending}
      aria-busy={pending || undefined}
      {...props}
    >
      {pending && (
        <span
          aria-hidden
          className="size-3.5 shrink-0 animate-spin rounded-full border-2 border-current border-r-transparent"
        />
      )}
      {children}
    </button>
  );
}
