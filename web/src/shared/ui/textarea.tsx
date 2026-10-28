import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Textarea({ className, ...props }: ComponentProps<"textarea">) {
  return (
    <textarea
      className={cn(
        "rounded-md border border-black/20 bg-transparent px-3 py-2 disabled:opacity-50 dark:border-white/20",
        className,
      )}
      {...props}
    />
  );
}
