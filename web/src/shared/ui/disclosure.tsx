import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { buttonVariants } from "./button";

type Props = {
  /** Texto do botão que abre e fecha. */
  summary: ReactNode;
  children: ReactNode;
};

/**
 * Bloco fechado atrás de um botão (`<details>` nativo, sem JavaScript), para
 * ações secundárias que só às vezes são usadas e ocupariam a tela toda.
 */
export function Disclosure({ summary, children }: Props) {
  return (
    <details className="flex flex-col">
      <summary
        className={cn(
          buttonVariants({ variant: "outline", size: "sm" }),
          "w-fit cursor-pointer list-none [&::-webkit-details-marker]:hidden",
        )}
      >
        {summary}
      </summary>
      <div className="mt-3 flex flex-col gap-2">{children}</div>
    </details>
  );
}
