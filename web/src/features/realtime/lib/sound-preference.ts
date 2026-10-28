/**
 * Se os avisos com o site aberto tocam som: ligado por padrão, a escolha fica
 * no `localStorage` deste navegador. Sem armazenamento (janela privada), vale
 * até fechar a página.
 */

const KEY = "notification-sound";
const listeners = new Set<() => void>();
let fallback = true;

export function soundEnabled(): boolean {
  try {
    return localStorage.getItem(KEY) !== "off";
  } catch {
    return fallback;
  }
}

export function setSoundEnabled(enabled: boolean) {
  fallback = enabled;
  try {
    if (enabled) localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, "off");
  } catch {
    // Fica só na memória.
  }
  for (const listener of listeners) listener();
}

/** Avisa quando a escolha muda (nesta aba ou em outra). */
export function subscribeSound(listener: () => void) {
  listeners.add(listener);
  const onStorage = (event: StorageEvent) => {
    if (event.key === KEY) listener();
  };
  window.addEventListener("storage", onStorage);
  return () => {
    listeners.delete(listener);
    window.removeEventListener("storage", onStorage);
  };
}
