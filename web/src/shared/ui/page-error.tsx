import type { ReactNode } from "react";

import { Link } from "@/i18n/navigation";
import { Alert } from "./alert";
import { buttonVariants } from "./button";

type Props = {
  /** O erro já traduzido. */
  children: ReactNode;
  /** Para onde dá para ir dali (a página de cima que ainda abre). */
  href: string;
  label: string;
};

/** Erro que ocupa a página (não encontrado, sem acesso), com o caminho de volta. */
export function PageError({ children, href, label }: Props) {
  return (
    <div className="flex flex-col items-start gap-3">
      <Alert>{children}</Alert>
      <Link
        href={href}
        className={buttonVariants({ variant: "outline", size: "sm" })}
      >
        {label}
      </Link>
    </div>
  );
}
