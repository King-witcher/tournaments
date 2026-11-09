import { createContext, useContext } from "react";

import type { ImageDraft } from "@/features/images/hooks/use-image-draft";
import type { components } from "@/shared/api/schema";
import type { TournamentValues, TranslationDraft } from "../lib/form-values";
import type { SettingsSection, ValuesSection } from "../lib/settings-sections";

type Locale = components["schemas"]["Locale"];

/** O que a pessoa mudou em cada seção e não salvou, por cima do salvo. */
export type ValuesEdits = Partial<TournamentValues>;
export type TranslationEdits = Partial<
  Record<Locale, Partial<TranslationDraft>>
>;

export type SettingsDraftsState = {
  /** As mudanças não salvas da seção (sempre o mesmo objeto sem mudança). */
  editsOf: (section: ValuesSection) => ValuesEdits;
  /** Troca as mudanças não salvas da seção; a mesma função em todo render. */
  setEdits: (
    section: ValuesSection,
    update: (edits: ValuesEdits) => ValuesEdits,
  ) => void;
  /** As mudanças não salvas de Outros idiomas. */
  translationEdits: TranslationEdits;
  /** Troca as de Outros idiomas; a mesma função em todo render. */
  setTranslationEdits: (
    update: (edits: TranslationEdits) => TranslationEdits,
  ) => void;
  /** A capa pendente de Geral, que sobrevive à troca de seção como os campos. */
  cover: ImageDraft;
  /** As seções com alguma mudança não salva, na ordem da página. */
  dirty: readonly SettingsSection[];
  /**
   * Vai à página da seção, com o foco no título dela: o "Ir para" de um erro
   * que nasce em outra seção. A mesma função enquanto o endereço não muda.
   */
  goTo: (section: SettingsSection) => void;
  /**
   * A seção acabou de abrir por um `goTo` para ela (e o pedido acaba aqui):
   * o título recebe o foco.
   */
  takeArrival: (section: SettingsSection) => boolean;
};

export const SettingsDraftsContext = createContext<SettingsDraftsState | null>(
  null,
);

/**
 * O que a pessoa mudou nas Configurações e não salvou
 * (`TournamentSettingsProvider`, no `layout.tsx` delas): sobrevive à troca de
 * seção, porque o layout não remonta, e some ao sair das Configurações.
 */
export function useSettingsDrafts() {
  const state = useContext(SettingsDraftsContext);
  if (!state) {
    throw new Error("useSettingsDrafts fora do TournamentSettingsProvider");
  }
  return state;
}
