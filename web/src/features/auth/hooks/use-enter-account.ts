import { useLocale } from "next-intl";

import { getPathname } from "@/i18n/navigation";

/**
 * Depois de entrar, carrega a página de destino do zero: o cabeçalho (que
 * fica no layout e a navegação do cliente não refaz) e o cache do roteador
 * ainda são de quem não estava logado.
 */
export function useEnterAccount() {
  const locale = useLocale();
  return (returnTo: string) =>
    window.location.assign(getPathname({ href: returnTo, locale }));
}
