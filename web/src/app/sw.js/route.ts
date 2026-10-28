import { serviceWorkerScript } from "@/features/push/lib/service-worker-script";
import { routing } from "@/i18n/routing";
import es from "../../../messages/es/Notifications.json";
import pt from "../../../messages/pt/Notifications.json";

/**
 * O service worker do Web Push em `/sw.js` (escopo `/`), com os textos dos
 * avisos de `messages/<idioma>/Notifications.json` embutidos. Estático (sai
 * no build); `no-cache` para o navegador conferir a versão a cada visita. O
 * proxy do next-intl não passa por aqui (caminho com ponto).
 */
export function GET() {
  const script = serviceWorkerScript(
    { pt: pt.notices, es: es.notices },
    routing.defaultLocale,
  );
  return new Response(script, {
    headers: {
      "Content-Type": "application/javascript; charset=utf-8",
      "Cache-Control": "no-cache",
      "Service-Worker-Allowed": "/",
    },
  });
}
