import { type RefObject, useState } from "react";

/**
 * Para um diálogo que só existe enquanto aberto: guarda o controle que tinha o
 * foco quando o diálogo apareceu (o botão que o abriu) e devolve uma função
 * que leva o foco de volta para ele. Sem isso, ao fechar, o diálogo sai da
 * árvore e o foco cai no `body`. `launcher` troca o controle guardado, para o
 * navegador que não dá foco ao botão no clique (o Safari).
 */
export function useReturnFocus(launcher?: RefObject<HTMLElement | null>) {
  const [opener] = useState(() => document.activeElement);
  return () =>
    requestAnimationFrame(() => {
      const target = launcher?.current ?? opener;
      if (target instanceof HTMLElement && target.isConnected) target.focus();
    });
}
