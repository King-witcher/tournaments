import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

/** Erro em destaque: o erro principal de uma página ou formulário. */
export function Alert({ children }: { children: ReactNode }) {
  return (
    <p
      role="alert"
      className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm"
    >
      {children}
    </p>
  );
}

/** Erro curto, junto do controle que falhou. */
export function ErrorText({
  children,
  size = "sm",
}: {
  children: ReactNode;
  size?: "sm" | "xs";
}) {
  return (
    <p
      role="alert"
      className={cn(
        "text-red-600 dark:text-red-400",
        size === "xs" ? "text-xs" : "text-sm",
      )}
    >
      {children}
    </p>
  );
}
