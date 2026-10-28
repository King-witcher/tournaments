/**
 * O service worker do Web Push, servido por `app/sw.js/route.ts` com os
 * textos dos avisos embutidos (`Notifications.notices` de cada idioma), para
 * o texto continuar só em `messages/`. JavaScript puro, sem build: os textos
 * usam só `{param}` (nada de plural do ICU), que o `fill` daqui troca.
 *
 * O que ele faz:
 * - `push`: lê `{code, params, path, locale}`. Com alguma janela do site
 *   visível, não mostra nada (a página já mostrou o toast pelo WebSocket);
 *   senão, a notificação no idioma da inscrição, com `tag` por código e
 *   caminho (o aviso repetido substitui o anterior, tocando de novo) e o
 *   endereço `/{locale}{path}`.
 * - `notificationclick`: foca a janela que já está nesse endereço ou abre
 *   uma nova.
 * - `install`/`activate`: a versão nova assume na hora.
 */
export function serviceWorkerScript(
  messages: Record<string, unknown>,
  defaultLocale: string,
) {
  return [
    "// Gerado por /sw.js (web/src/features/push/lib/service-worker-script.ts).",
    '"use strict";',
    `const MESSAGES = ${JSON.stringify(messages)};`,
    `const DEFAULT_LOCALE = ${JSON.stringify(defaultLocale)};`,
    BODY,
  ].join("\n");
}

const BODY = String.raw`
self.addEventListener("install", () => {
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

function fill(template, values) {
  return template.replace(/\{(\w+)\}/g, (all, name) =>
    values[name] === undefined || values[name] === null ? all : String(values[name]),
  );
}

// O prazo no fuso de quem vê: só a hora se for hoje (como formatDeadline).
function deadline(value, locale) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return String(value);
  const today = date.toDateString() === new Date().toDateString();
  return new Intl.DateTimeFormat(
    locale,
    today
      ? { hour: "2-digit", minute: "2-digit" }
      : { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" },
  ).format(date);
}

function text(notice) {
  const locale = Object.prototype.hasOwnProperty.call(MESSAGES, notice.locale)
    ? notice.locale
    : DEFAULT_LOCALE;
  const [group, name] = String(notice.code).split(".");
  const entry = MESSAGES[locale][group] && MESSAGES[locale][group][name];
  if (!entry) return undefined;
  const values = Object.assign({}, notice.params);
  if (values.confirm_by) values.deadline = deadline(values.confirm_by, locale);
  return { locale, title: fill(entry.title, values), body: fill(entry.body, values) };
}

async function windows() {
  return self.clients.matchAll({ type: "window", includeUncontrolled: true });
}

async function show(notice) {
  if ((await windows()).some((w) => w.visibilityState === "visible")) return;
  const content = text(notice);
  if (!content) return;
  const path =
    typeof notice.path === "string" && notice.path.startsWith("/") ? notice.path : "/";
  await self.registration.showNotification(content.title, {
    body: content.body,
    tag: notice.code + " " + path,
    renotify: true,
    icon: "/icon",
    lang: content.locale,
    data: { url: "/" + content.locale + path },
  });
}

self.addEventListener("push", (event) => {
  let notice;
  try {
    notice = event.data ? event.data.json() : undefined;
  } catch {
    notice = undefined;
  }
  if (notice) event.waitUntil(show(notice));
});

async function focusOrOpen(url) {
  for (const client of await windows()) {
    if (client.url === url && "focus" in client) return client.focus();
  }
  return self.clients.openWindow(url);
}

self.addEventListener("notificationclick", (event) => {
  event.notification.close();
  const data = event.notification.data || {};
  const url = new URL(data.url || "/", self.location.origin).href;
  event.waitUntil(focusOrOpen(url));
});
`;
