import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

/** Coluna central do conteúdo de uma página. */
export function Page({
  children,
  spacing = "normal",
}: {
  children: ReactNode;
  spacing?: "normal" | "loose";
}) {
  return (
    <main
      className={cn(
        "mx-auto flex w-full max-w-2xl flex-1 flex-col px-4 py-16",
        spacing === "loose" ? "gap-8" : "gap-4",
      )}
    >
      {children}
    </main>
  );
}
