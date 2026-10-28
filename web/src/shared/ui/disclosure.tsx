import type { ReactNode } from "react";

import { Accordion } from "./accordion";

type Props = {
  /** Texto do que abre e fecha. */
  summary: ReactNode;
  children: ReactNode;
};

/** @deprecated Use `Accordion` (`title` no lugar de `summary`). */
export function Disclosure({ summary, children }: Props) {
  return <Accordion title={summary}>{children}</Accordion>;
}
