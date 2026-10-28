import createMiddleware from "next-intl/middleware";

import { routing } from "./i18n/routing";

export default createMiddleware(routing);

export const config = {
  // Tudo, exceto rotas internas do Next/Vercel, os ícones gerados (`/icon`,
  // `/apple-icon`, sem extensão) e arquivos com extensão (`/sw.js`,
  // `/manifest.webmanifest`), que não têm idioma.
  matcher: "/((?!api|_next|_vercel|icon$|apple-icon$|.*\\..*).*)",
};
