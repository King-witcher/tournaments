import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Textarea({ className, ...props }: ComponentProps<"textarea">) {
  return (
    <textarea
      className={cn(
        "min-h-11 rounded-md border border-black/40 bg-transparent px-3 py-2 disabled:opacity-50 dark:border-white/40",
        className,
      )}
      {...props}
    />
  );
}
