import createMiddleware from "next-intl/middleware";

import { routing } from "./i18n/routing";

export default createMiddleware(routing);

export const config = {
  // Tudo, exceto rotas internas do Next/Vercel e arquivos com extensão.
  matcher: "/((?!api|_next|_vercel|.*\\..*).*)",
};
