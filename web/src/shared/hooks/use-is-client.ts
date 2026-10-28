import { useSyncExternalStore } from "react";

const subscribe = () => () => {};

/**
 * `false` no servidor e durante a hidratação, `true` depois. Para o que
 * depende do navegador (o fuso de quem vê, por exemplo) sem divergir do HTML
 * que veio do servidor.
 */
export function useIsClient() {
  return useSyncExternalStore(
    subscribe,
    () => true,
    () => false,
  );
}
