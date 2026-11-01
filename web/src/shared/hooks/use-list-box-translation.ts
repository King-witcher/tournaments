import type {
  ListBoxMenuIconTranslationKey,
  ListBoxSelectionTranslationKey,
} from "@carbon/react";
import { useTranslations } from "next-intl";

type Key = ListBoxMenuIconTranslationKey | ListBoxSelectionTranslationKey;

// Os ids que o Carbon passa ao `translateWithId` e a chave de cada um em
// `Common.listBox`.
const KEYS = {
  "open.menu": "openMenu",
  "close.menu": "closeMenu",
  "clear.selection": "clearSelection",
  "clear.all": "clearAll",
} as const satisfies Record<Key, string>;

/**
 * O `translateWithId` dos campos de lista do Carbon (`Dropdown`, `ComboBox`,
 * `MultiSelect`): os rótulos de abrir e fechar o menu e de limpar a seleção,
 * que sem ele saem em inglês.
 */
export function useListBoxTranslation() {
  const t = useTranslations("Common.listBox");
  return (id: Key) => t(KEYS[id]);
}
