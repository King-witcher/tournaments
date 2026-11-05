import {
  type NextFetchEvent,
  type NextRequest,
  NextResponse,
} from "next/server";
import { hasLocale } from "next-intl";
import createMiddleware from "next-intl/middleware";

import { SESSION_COOKIE } from "./features/auth/lib/session-cookie";
import { refuseScanner } from "./features/scanners/lib/refuse-scanner";
import { routing } from "./i18n/routing";

const intl = createMiddleware(routing);

/** A rota do início de quem entrou (`app/[locale]/me`), servida em `/`. */
const SIGNED_IN_HOME = "me";

/** O nome padrão do next-intl. */
const LOCALE_COOKIE = "NEXT_LOCALE";

/**
 * O que tem idioma: tudo, exceto rotas internas do Next/Vercel, os ícones gerados
 * (`/icon`, `/apple-icon`, sem extensão) e arquivos com extensão (`/sw.js`,
 * `/manifest.webmanifest`).
 */
const HAS_LOCALE = /^\/(?!api|_next|_vercel|icon$|apple-icon$|.*\..*)/;

/**
 * Primeiro, os scanners de vulnerabilidade (`refuseScanner`), e por isso o proxy
 * também roda nos caminhos sem idioma.
 *
 * Depois, o idioma (next-intl) e, no início, quem vê o quê. Sem o cookie de sessão,
 * a apresentação do site, estática (`app/[locale]/(home)`), sai pronta sem
 * esperar nada; com ele, o mesmo endereço mostra o início de quem entrou
 * (`/me`, que não aparece na barra: aberto direto, volta para `/`). Aqui não se
 * pergunta à API, para a apresentação não esperar por ela: cookie de sessão
 * vencida também cai em `/me`, e sai de lá (`ClearExpiredSession`), o que traz a
 * apresentação.
 */
export default function proxy(request: NextRequest, event: NextFetchEvent) {
  const refused = refuseScanner(request, event);
  if (refused) {
    return refused;
  }
  const { pathname, search } = request.nextUrl;
  if (!HAS_LOCALE.test(pathname)) {
    return NextResponse.next();
  }
  const [, locale, segment, ...rest] = pathname.split("/");
  if (hasLocale(routing.locales, locale) && rest.length === 0) {
    if (segment === SIGNED_IN_HOME) {
      return NextResponse.redirect(new URL(`/${locale}${search}`, request.url));
    }
    if (!segment && request.cookies.has(SESSION_COOKIE)) {
      const response = NextResponse.rewrite(
        new URL(`/${locale}/${SIGNED_IN_HOME}${search}`, request.url),
      );
      // Esta resposta não passa pelo next-intl, que guarda o idioma aberto.
      if (request.cookies.get(LOCALE_COOKIE)?.value !== locale) {
        response.cookies.set(LOCALE_COOKIE, locale, { sameSite: "lax" });
      }
      return response;
    }
  }
  return intl(request);
}

export const config = {
  // Tudo, exceto o que é pedido às dezenas por página e só um endereço exato
  // alcança: os arquivos do build, as imagens otimizadas e as imagens e os
  // sons de `public/`.
  matcher:
    "/((?!_next/static|_next/image|_vercel|icon$|apple-icon$|.*\\.webp$|.*\\.mp3$).*)",
};
