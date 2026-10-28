import type { Messages } from "./i18n/namespaces";
import type { routing } from "./i18n/routing";

// Tipa as chaves de tradução e os idiomas: chave inexistente vira erro de compilação.
declare module "next-intl" {
  interface AppConfig {
    Locale: (typeof routing.locales)[number];
    Messages: Messages;
  }
}
