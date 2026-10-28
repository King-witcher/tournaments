import { Stack } from "@carbon/react";
import type { ReactNode } from "react";

import { Link } from "./link";
import { Notice } from "./notice";

type Props = {
  /** O erro já traduzido. */
  children: ReactNode;
  /** Para onde dá para ir dali (a página de cima que ainda abre). */
  href: string;
  label: string;
};

/**
 * Erro que ocupa a página (não encontrado, sem acesso): o aviso e o caminho
 * de volta como link.
 */
export function PageError({ children, href, label }: Props) {
  return (
    <Stack gap={5}>
      <Notice kind="error">{children}</Notice>
      <Link href={href}>{label}</Link>
    </Stack>
  );
}
