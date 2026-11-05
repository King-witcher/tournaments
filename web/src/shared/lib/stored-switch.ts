/** Uma escolha liga/desliga deste navegador (`storedSwitch`). */
export type StoredSwitch = {
  get: () => boolean;
  set: (enabled: boolean) => void;
  /** Avisa quando a escolha muda, nesta aba ou em outra; devolve a função que desfaz. */
  subscribe: (listener: () => void) => () => void;
};

/**
 * Uma escolha liga/desliga, ligada por padrão, guardada no `localStorage`
 * deste navegador em `key` (desligada fica `"off"`). Sem armazenamento (janela
 * privada), vale até fechar a página.
 */
export function storedSwitch(key: string): StoredSwitch {
  const listeners = new Set<() => void>();
  let fallback = true;
  return {
    get() {
      try {
        return localStorage.getItem(key) !== "off";
      } catch {
        return fallback;
      }
    },
    set(enabled) {
      fallback = enabled;
      try {
        if (enabled) localStorage.removeItem(key);
        else localStorage.setItem(key, "off");
      } catch {
        // Fica só na memória.
      }
      for (const listener of listeners) listener();
    },
    subscribe(listener) {
      listeners.add(listener);
      const onStorage = (event: StorageEvent) => {
        if (event.key === key) listener();
      };
      window.addEventListener("storage", onStorage);
      return () => {
        listeners.delete(listener);
        window.removeEventListener("storage", onStorage);
      };
    },
  };
}
