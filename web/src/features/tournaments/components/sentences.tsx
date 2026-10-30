import { Fragment, type ReactNode } from "react";

/**
 * Frases que seguem a configuração, juntas num parágrafo ou item: as vazias
 * (`false`, `null`, `""`) saem, e as outras ficam separadas por um espaço.
 */
export function Sentences({ children }: { children: ReactNode[] }) {
  return children.filter(Boolean).map((sentence, index) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: frases fixas, na ordem.
    <Fragment key={index}>
      {index > 0 && " "}
      {sentence}
    </Fragment>
  ));
}
