import { type NextRequest, NextResponse } from "next/server";
import { hasLocale } from "next-intl";
import createMiddleware from "next-intl/middleware";

import { SESSION_COOKIE } from "./features/auth/lib/session-cookie";
import { routing } from "./i18n/routing";

const intl = createMiddleware(routing);

/** A rota do início de quem entrou (`app/[locale]/me`), servida em `/`. */
const SIGNED_IN_HOME = "me";

/**
 * O idioma (next-intl) e, no início, quem vê o quê. Sem o cookie de sessão,
 * a apresentação do site, estática (`app/[locale]/(home)`), sai pronta sem
 * esperar nada; com ele, o mesmo endereço mostra o início de quem entrou
 * (`/me`, que não aparece na barra: aberto direto, volta para `/`). Cookie de
 * sessão vencida também cai em `/me`, que convida a entrar de novo.
 */
export default function proxy(request: NextRequest) {
  const { pathname, search } = request.nextUrl;
  const [, locale, segment, ...rest] = pathname.split("/");
  if (hasLocale(routing.locales, locale) && rest.length === 0) {
    if (segment === SIGNED_IN_HOME) {
      return NextResponse.redirect(new URL(`/${locale}${search}`, request.url));
    }
    if (!segment && request.cookies.has(SESSION_COOKIE)) {
      return NextResponse.rewrite(
        new URL(`/${locale}/${SIGNED_IN_HOME}${search}`, request.url),
      );
    }
  }
  return intl(request);
}

export const config = {
  // Tudo, exceto rotas internas do Next/Vercel, os ícones gerados (`/icon`,
  // `/apple-icon`, sem extensão) e arquivos com extensão (`/sw.js`,
  // `/manifest.webmanifest`), que não têm idioma.
  matcher: "/((?!api|_next|_vercel|icon$|apple-icon$|.*\\..*).*)",
};
