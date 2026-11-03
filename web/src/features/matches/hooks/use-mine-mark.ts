"use client";

import { useRef, useState } from "react";

/** Quanto tempo uma ação de quem vê espera a mudança que ela causa. */
const MINE_MS = 10_000;

/**
 * Marca a próxima mudança da tela como de quem vê, para os passos seguirem
 * para o novo só nesse caso. A hora da última ação fica numa ref (0: nenhuma
 * por atender), e as funções são as mesmas entre as renderizações (entram em
 * efeitos).
 */
export function useMineMark() {
  const mineAt = useRef(0);
  const [mine] = useState(() => ({
    markMine: () => {
      mineAt.current = Date.now();
    },
    isMine: () => Date.now() - mineAt.current < MINE_MS,
    clearMine: () => {
      mineAt.current = 0;
    },
  }));
  return mine;
}
