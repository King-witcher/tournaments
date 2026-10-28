import type { ReactNode } from "react";

import { Link } from "@/shared/ui/link";
import { memberPath } from "../lib/paths";

type Props = {
  tenant: string;
  userId: number;
  /** O apelido (e o que mais for junto, como o avatar). */
  children: ReactNode;
  /** No meio de uma frase: sublinhado, sem a altura de toque. */
  inline?: boolean;
  size?: "sm" | "md" | "lg";
  className?: string;
};

/** O apelido como link para a página da pessoa na comunidade. */
export function MemberLink({
  tenant,
  userId,
  children,
  inline,
  size,
  className,
}: Props) {
  return (
    <Link
      href={memberPath(tenant, userId)}
      inline={inline}
      size={size}
      className={className}
    >
      {children}
    </Link>
  );
}
