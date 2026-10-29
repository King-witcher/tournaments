import { type RefObject, useRef, useState } from "react";

/**
 * Para abrir um `FormModal` a partir de um item do `OverflowActions`: o
 * `wrapper` vai num elemento em volta do menu, e `trigger` aponta para o
 * botão ⋯ dele (o `returnFocusTo` do Modal), para o foco voltar ao menu
 * quando o Modal fecha sem enviar.
 */
export function useMenuTrigger() {
  const wrapper = useRef<HTMLDivElement>(null);
  const [trigger] = useState<RefObject<HTMLElement | null>>(() => ({
    get current() {
      return wrapper.current?.querySelector("button") ?? null;
    },
  }));
  return { wrapper, trigger };
}
