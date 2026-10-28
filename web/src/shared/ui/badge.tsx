import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

const variants = {
  neutral: "bg-black/5 dark:bg-white/10",
  accent: "bg-blue-600 text-white dark:bg-blue-500",
  warning: "bg-amber-500/20 text-amber-900 dark:text-amber-200",
  success: "bg-green-600/15 text-green-800 dark:text-green-300",
};

/** Etiqueta curta de estado, em pílula. */
export function Badge({
  children,
  variant = "neutral",
}: {
  children: ReactNode;
  variant?: keyof typeof variants;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium",
        variants[variant],
      )}
    >
      {children}
    </span>
  );
}
