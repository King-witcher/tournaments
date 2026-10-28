/**
 * A inscrição de Web Push deste navegador: o service worker (`/sw.js`, escopo
 * `/`), a inscrição no serviço de push e o que já foi mandado à API (no
 * `localStorage`, para mandar de novo só quando muda o idioma ou a conta).
 * Só no navegador.
 */

import type { components } from "@/shared/api/schema";
import { pushSupport } from "./push-support";

type PutSubscription = components["schemas"]["PutSubscription"];
type Locale = PutSubscription["locale"];

const SAVED_KEY = "push-subscription";

/** A inscrição atual deste navegador, se houver. Não registra o service worker. */
export async function currentSubscription() {
  const registration = await navigator.serviceWorker.getRegistration("/");
  return (await registration?.pushManager.getSubscription()) ?? null;
}

/**
 * Se os avisos chegam a este navegador por Web Push (permissão dada e
 * inscrito): com a aba escondida, a notificação do sistema já avisa.
 */
export async function receivesPush() {
  if (pushSupport() !== "supported") return false;
  if (Notification.permission !== "granted") return false;
  return (await currentSubscription().catch(() => null)) !== null;
}

/**
 * Registra o service worker e inscreve o navegador com a chave VAPID da API
 * (a inscrição que já existe com outra chave sai antes). Precisa da
 * permissão de notificação já dada.
 */
export async function subscribeBrowser(publicKey: string) {
  const registration = await navigator.serviceWorker.register("/sw.js", {
    scope: "/",
    updateViaCache: "none",
  });
  await navigator.serviceWorker.ready;
  const key = base64UrlToBytes(publicKey);
  const existing = await registration.pushManager.getSubscription();
  if (existing) {
    if (sameBytes(existing.options.applicationServerKey, key)) return existing;
    await existing.unsubscribe();
  }
  return registration.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: key,
  });
}

/** O corpo de `PUT /push/subscriptions`: a inscrição e o idioma dos avisos. */
export function subscriptionBody(
  subscription: PushSubscription,
  locale: Locale,
): PutSubscription {
  const { endpoint, keys } = subscription.toJSON();
  return {
    endpoint: endpoint ?? subscription.endpoint,
    keys: { p256dh: keys?.p256dh ?? "", auth: keys?.auth ?? "" },
    locale,
  };
}

type Saved = { endpoint: string; locale: string; user?: number };

/** O que foi mandado à API por último deste navegador. */
export function savedSubscription(): Saved | undefined {
  try {
    const value = localStorage.getItem(SAVED_KEY);
    return value ? (JSON.parse(value) as Saved) : undefined;
  } catch {
    return undefined;
  }
}

export function saveSubscription(saved: Saved | undefined) {
  try {
    if (saved) localStorage.setItem(SAVED_KEY, JSON.stringify(saved));
    else localStorage.removeItem(SAVED_KEY);
  } catch {
    // Sem armazenamento: o PUT sai de novo a cada carga, sem outro efeito.
  }
}

/** base64url (a chave pública VAPID) em bytes, como o `subscribe` pede. */
function base64UrlToBytes(value: string) {
  const padded = value.replace(/-/g, "+").replace(/_/g, "/");
  const binary = atob(padded + "=".repeat((4 - (padded.length % 4)) % 4));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

function sameBytes(a: ArrayBuffer | null, b: Uint8Array) {
  if (!a || a.byteLength !== b.length) return false;
  const view = new Uint8Array(a);
  return view.every((byte, i) => byte === b[i]);
}
