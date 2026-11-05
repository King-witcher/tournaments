"use client";

import { useVetoSounds } from "../hooks/use-veto-sounds";

/**
 * Os sons do veto, sem nada na tela: fica junto do `VetoBoardContext` (da
 * série e da sala), montado o tempo todo, e não nos passos, que só existem
 * enquanto estão à vista.
 */
export function VetoSounds() {
  useVetoSounds();
  return null;
}
