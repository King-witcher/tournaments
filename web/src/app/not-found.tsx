import "./globals.scss";

import { StandaloneErrorDocument } from "@/features/errors/components/standalone-error-document";

/**
 * Endereço que o proxy do idioma não atende (`/api/...`, um arquivo com
 * extensão): o primeiro segmento vira `[locale]`, o layout não o reconhece e
 * lança `notFound()`, que chega aqui, acima do layout; por isso o documento
 * inteiro. Sem idioma no endereço, os dois textos, português primeiro. (O
 * `[locale]/not-found.tsx` cobre o resto, dentro do layout.)
 */
export default function RootNotFound() {
  return <StandaloneErrorDocument locales={["pt", "es"]} kind="notFound" />;
}
