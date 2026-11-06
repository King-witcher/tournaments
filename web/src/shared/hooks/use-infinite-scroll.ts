import { useEffect, useRef, useState } from "react";

/** A área de rolagem mais próxima de `element` (um painel); sem ela, a página. */
function scrollParent(element: HTMLElement) {
  for (let node = element.parentElement; node; node = node.parentElement) {
    const { overflowY } = getComputedStyle(node);
    if (overflowY === "auto" || overflowY === "scroll") return node;
  }
  return null;
}

/**
 * Rolagem infinita: devolve o `ref` de um marcador no fim da lista e chama
 * `onReach` quando ele chega a uma altura da área de rolagem dele (o painel
 * ou a página), antes de aparecer, para a página seguinte já estar lá. Com
 * `enabled` falso (buscando, sem mais páginas, depois de um erro), não
 * observa; ao voltar a true, confere de novo, então uma página que não encheu
 * a área já traz a seguinte.
 */
export function useInfiniteScroll(onReach: () => void, enabled: boolean) {
  const [marker, setMarker] = useState<HTMLElement | null>(null);
  const reach = useRef(onReach);
  useEffect(() => {
    reach.current = onReach;
  });

  useEffect(() => {
    if (!enabled || !marker) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) reach.current();
      },
      { root: scrollParent(marker), rootMargin: "0px 0px 100% 0px" },
    );
    observer.observe(marker);
    return () => observer.disconnect();
  }, [enabled, marker]);

  return setMarker;
}
