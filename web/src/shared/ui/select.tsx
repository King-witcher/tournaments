import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

/** `<select>` nativo com o visual do `Input`. */
export function Select({ className, ...props }: ComponentProps<"select">) {
  return (
    <select
      className={cn(
        "min-h-11 rounded-md border border-black/40 bg-background px-3 py-2 disabled:opacity-50 dark:border-white/40",
        className,
      )}
      {...props}
    />
  );
}
