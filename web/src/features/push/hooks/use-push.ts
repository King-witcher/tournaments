import { useLocale, useTranslations } from "next-intl";
import { useEffect, useState, useTransition } from "react";

import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import {
  currentSubscription,
  saveSubscription,
  subscribeBrowser,
  subscriptionBody,
} from "../lib/browser-subscription";
import { pushKey } from "../lib/push-key";
import { pushSupport } from "../lib/push-support";

/**
 * O Web Push neste navegador: `loading` até conferir; `unsupported`;
 * `install` (iPhone fora do site instalado); `server_off` (a API está sem as
 * chaves); `denied` (bloqueado nas configurações do navegador); `off`; `on`.
 */
export type PushStatus =
  | "loading"
  | "unsupported"
  | "install"
  | "server_off"
  | "denied"
  | "off"
  | "on";

async function detect(): Promise<{ status: PushStatus; asked: boolean }> {
  const support = pushSupport();
  if (support !== "supported") return { status: support, asked: false };
  const permission = Notification.permission;
  if (permission === "denied") return { status: "denied", asked: true };
  const asked = permission === "granted";
  if (asked && (await currentSubscription())) return { status: "on", asked };
  const key = await pushKey();
  return { status: key.kind === "off" ? "server_off" : "off", asked };
}

/**
 * Estado e ações do Web Push deste navegador. `asked`: a permissão já foi
 * respondida (o convite a ativar só aparece antes disso). `enable` pede a
 * permissão (sempre num clique: o navegador só pergunta dentro do gesto),
 * registra o service worker, inscreve e manda a inscrição à API com o idioma
 * da página; `disable` tira da API e do navegador.
 */
export function usePush() {
  const t = useTranslations("Notifications.push");
  const locale = useLocale();
  const errorMessage = useApiErrorMessage();
  const [status, setStatus] = useState<PushStatus>("loading");
  const [asked, setAsked] = useState(false);
  const [error, setError] = useState<string>();
  const [pending, startTransition] = useTransition();

  useEffect(() => {
    let live = true;
    detect()
      .catch(() => ({ status: "unsupported" as const, asked: false }))
      .then((found) => {
        if (!live) return;
        setStatus(found.status);
        setAsked(found.asked);
      });
    return () => {
      live = false;
    };
  }, []);

  const enable = () => {
    setError(undefined);
    // Pedida já no clique, antes de qualquer espera: o Safari só pergunta
    // dentro do gesto.
    const permission = Notification.requestPermission();
    startTransition(async () => {
      const answer = await permission;
      setAsked(true);
      if (answer !== "granted") {
        setStatus(answer === "denied" ? "denied" : "off");
        return;
      }
      const key = await pushKey();
      if (key.kind !== "key") {
        if (key.kind === "off") setStatus("server_off");
        else setError(errorMessage(key.code));
        return;
      }
      try {
        const subscription = await subscribeBrowser(key.value);
        const outcome = await callApi(
          api.PUT("/push/subscriptions", {
            body: subscriptionBody(subscription, locale),
          }),
        );
        if (!outcome.ok) {
          setError(errorMessage(outcome.code));
          return;
        }
        saveSubscription({ endpoint: subscription.endpoint, locale });
        setStatus("on");
      } catch {
        setError(t("failed"));
      }
    });
  };

  const disable = () => {
    setError(undefined);
    startTransition(async () => {
      try {
        const subscription = await currentSubscription();
        if (subscription) {
          // Sem a API, sair do navegador basta: o serviço de push passa a
          // responder que a inscrição não existe, e a API a apaga.
          await callApi(
            api.DELETE("/push/subscriptions", {
              body: { endpoint: subscription.endpoint },
            }),
          );
          await subscription.unsubscribe();
        }
        saveSubscription(undefined);
        setStatus("off");
      } catch {
        setError(t("failed"));
      }
    });
  };

  return { status, asked, pending, error, enable, disable };
}
