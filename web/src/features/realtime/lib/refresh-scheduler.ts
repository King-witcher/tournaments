/** O primeiro aviso espera só o bastante para juntar os da mesma mudança. */
const FIRST_DELAY_MS = 300;
/** Depois de uma recarga, os avisos desta janela viram uma recarga só. */
const WINDOW_MS = 1500;

const hidden = () => document.visibilityState === "hidden";

/**
 * Junta os pedidos de recarga: o primeiro sai em ~0,3 s e os que chegam até
 * 1,5 s depois da última recarga viram uma só. Com a aba escondida não
 * recarrega (cada recarga lê a sessão na API, e a cota do Redis é contada):
 * guarda que ficou desatualizada e recarrega uma vez ao voltar a ser vista.
 */
export function createRefreshScheduler(refresh: () => void) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let last = Number.NEGATIVE_INFINITY;
  let stale = false;

  const fire = () => {
    timer = undefined;
    if (hidden()) {
      stale = true;
      return;
    }
    last = Date.now();
    refresh();
  };

  const request = () => {
    if (hidden()) {
      stale = true;
      return;
    }
    if (timer !== undefined) return;
    const wait = Math.max(FIRST_DELAY_MS, last + WINDOW_MS - Date.now());
    timer = setTimeout(fire, wait);
  };

  const onVisibility = () => {
    if (hidden() || !stale) return;
    stale = false;
    request();
  };
  document.addEventListener("visibilitychange", onVisibility);

  return {
    request,
    dispose() {
      clearTimeout(timer);
      document.removeEventListener("visibilitychange", onVisibility);
    },
  };
}
