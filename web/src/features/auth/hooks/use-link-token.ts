import { useSyncExternalStore } from "react";

function subscribe(onChange: () => void) {
  window.addEventListener("hashchange", onChange);
  return () => window.removeEventListener("hashchange", onChange);
}

const readToken = () => window.location.hash.slice(1).trim();

const notYet = () => undefined;

/**
 * O token de um link mandado por email, que vem depois do `#`
 * (`/signup/confirm#TOKEN`): o fragmento não vai ao servidor nem aos logs, e
 * a página continua estática. `undefined` no servidor e durante a hidratação
 * (ainda não dá para saber); depois, o token, ou `""` se o link chegou sem
 * ele (cortado ao copiar).
 */
export function useLinkToken() {
  return useSyncExternalStore<string | undefined>(subscribe, readToken, notYet);
}
