/**
 * Se a pessoa dispensou a sugestão de ativar as notificações, neste navegador.
 * Fica no `localStorage`; sem armazenamento (janela privada), vale até
 * recarregar a página.
 */

const KEY = "push-suggestion-dismissed";

export function suggestionDismissed(): boolean {
  try {
    return localStorage.getItem(KEY) === "1";
  } catch {
    return false;
  }
}

export function dismissSuggestion() {
  try {
    localStorage.setItem(KEY, "1");
  } catch {
    // Só some desta vez.
  }
}
