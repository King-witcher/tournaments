import type messages from "../messages/pt.json";
import type { routing } from "./i18n/routing";

// Tipa as chaves de tradução e os idiomas: chave inexistente vira erro de compilação.
declare module "next-intl" {
  interface AppConfig {
    Locale: (typeof routing.locales)[number];
    Messages: typeof messages;
  }
}
