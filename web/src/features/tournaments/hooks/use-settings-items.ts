import { createContext, useContext } from "react";

import type { SettingsSection } from "../lib/settings-sections";

export type SettingsItemsState = {
  /** O item da seção está aberto. */
  isOpen: (section: SettingsSection) => boolean;
  /** Abre ou fecha o item (o clique no título). */
  setOpen: (section: SettingsSection, open: boolean) => void;
  /**
   * Abre o item, rola até ele e leva o foco ao título: o "Ir para" de um
   * erro que nasce em outra seção e o `#` do endereço. A mesma função em
   * todo render.
   */
  reveal: (section: SettingsSection) => void;
};

export const SettingsItemsContext = createContext<SettingsItemsState | null>(
  null,
);

/**
 * Quais itens da aba Configurações estão abertos (`TournamentSettingsProvider`).
 * Vários podem ficar abertos; todos começam fechados, menos o do `#`.
 */
export function useSettingsItems() {
  const state = useContext(SettingsItemsContext);
  if (!state) {
    throw new Error("useSettingsItems fora do TournamentSettingsProvider");
  }
  return state;
}
