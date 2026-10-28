import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Input({ className, ...props }: ComponentProps<"input">) {
  return (
    <input
      className={cn(
        "min-h-11 rounded-md border border-black/40 bg-transparent px-3 py-2 placeholder:text-neutral-500 dark:placeholder:text-neutral-400 disabled:opacity-50 dark:border-white/40",
        className,
      )}
      {...props}
    />
  );
}
