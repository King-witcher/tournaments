import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Input({ className, ...props }: ComponentProps<"input">) {
  return (
    <input
      className={cn(
        "rounded-md border border-black/20 bg-transparent px-3 py-2 dark:border-white/20",
        className,
      )}
      {...props}
    />
  );
}
