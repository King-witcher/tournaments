import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

// Nomes de variante e tamanho iguais aos do shadcn, para trocar sem mexer em quem usa.
const variants = {
  default:
    "bg-black font-medium text-white hover:bg-black/80 dark:bg-white dark:text-black dark:hover:bg-white/80",
  outline:
    "border border-black/15 hover:bg-black/5 dark:border-white/20 dark:hover:bg-white/10",
  link: "underline-offset-4 hover:underline",
};

const sizes = {
  default: "px-4 py-2",
  sm: "px-3 py-1.5",
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
    "rounded-md text-sm disabled:opacity-50",
    variants[variant],
    variant !== "link" && sizes[size],
  );
}

export function Button({
  variant,
  size,
  className,
  type = "button",
  ...props
}: ComponentProps<"button"> & ButtonStyle) {
  return (
    <button
      type={type}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}
