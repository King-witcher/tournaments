import type { ReactNode } from "react";

import { Link } from "@/i18n/navigation";

type Props = {
  href: string;
  /** O nome do item: sublinhado e na cor de link, para ver que abre algo. */
  title: ReactNode;
  /** Metadado (status, data, papel), em texto secundário. */
  children?: ReactNode;
};

/**
 * Item de lista que abre uma página: a linha inteira é o link, com pelo menos
 * 44px de altura (toque); no celular o metadado vai para baixo do nome.
 */
export function RowLink({ href, title, children }: Props) {
  return (
    <Link
      href={href}
      className="group flex min-h-11 flex-col justify-center gap-0.5 py-2 sm:flex-row sm:items-center sm:justify-between sm:gap-4"
    >
      <span className="font-medium text-link underline underline-offset-4 group-hover:decoration-2">
        {title}
      </span>
      {children && (
        <span className="text-sm text-muted sm:text-right">{children}</span>
      )}
    </Link>
  );
}
