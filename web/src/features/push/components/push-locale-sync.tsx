"use client";

import { useLocale } from "next-intl";
import { useEffect } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import {
  currentSubscription,
  savedSubscription,
  saveSubscription,
  subscriptionBody,
} from "../lib/browser-subscription";
import { pushSupport } from "../lib/push-support";

/**
 * Mantém a inscrição deste navegador com o idioma da página e a conta de
 * quem está logado: abrir o site em outro idioma, ou entrar com outra conta,
 * manda o `PUT` de novo (a inscrição passa a ser desta conta, e os avisos de
 * quem saiu deixam de chegar aqui). Não pede permissão nem registra nada.
 */
export function PushLocaleSync({ userId }: { userId: number }) {
  const locale = useLocale();
  useEffect(() => {
    if (pushSupport() !== "supported" || Notification.permission !== "granted")
      return;
    let live = true;
    (async () => {
      const subscription = await currentSubscription();
      if (!subscription || !live) return;
      const saved = savedSubscription();
      if (
        saved?.endpoint === subscription.endpoint &&
        saved.locale === locale &&
        saved.user === userId
      )
        return;
      const outcome = await callApi(
        api.PUT("/push/subscriptions", {
          body: subscriptionBody(subscription, locale),
        }),
      );
      if (outcome.ok) {
        saveSubscription({
          endpoint: subscription.endpoint,
          locale,
          user: userId,
        });
      }
    })().catch(() => {});
    return () => {
      live = false;
    };
  }, [locale, userId]);
  return null;
}
