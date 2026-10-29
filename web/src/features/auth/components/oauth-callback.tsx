"use client";

import { InlineLoading } from "@carbon/react";
import { type Locale, useLocale, useTranslations } from "next-intl";
import { useEffect, useRef } from "react";

import { getPathname } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { DELETION_CANCELLED } from "../lib/deletion-cancelled";
import { takeOAuthFlow } from "../lib/oauth-flow";
import type { Provider } from "../lib/providers";

/**
 * A volta do provedor (`?code=..&state=..`, ou `?error=..`): tira o code da
 * barra de endereço, pega o fluxo aberto nesta aba (`takeOAuthFlow`, que o
 * apaga: vale uma vez) e confere o provedor e o state. Batendo, manda o code
 * com o verifier à API, que o troca no provedor. Entrando, carrega o destino
 * do zero (o cabeçalho ainda é de quem não estava logado), ou a conta, se
 * entrar cancelou a exclusão; vinculando, volta para a página de origem.
 * Errado, volta com `?auth_error=<código>`: para a origem, ao vincular; para
 * o início, ao entrar. O callback sai do histórico (`replace`).
 */
export function OAuthCallback({ provider }: { provider: Provider }) {
  const t = useTranslations("Auth");
  const locale = useLocale();
  // No Strict Mode (dev), o efeito roda duas vezes; o fluxo só vale uma.
  const started = useRef(false);

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    finish(provider, locale).then((path) => window.location.replace(path));
  }, [provider, locale]);

  return <InlineLoading description={t("oauthCallback")} />;
}

/** Conclui o fluxo e diz para onde ir (já com o idioma). */
async function finish(provider: Provider, pageLocale: Locale) {
  const query = new URLSearchParams(window.location.search);
  // O code não fica no histórico nem no Referer do que vier depois.
  window.history.replaceState(null, "", window.location.pathname);
  const flow = takeOAuthFlow();
  const locale = flow?.locale ?? pageLocale;
  const to = (path: string) => getPathname({ href: path, locale });
  const fail = (code: string) =>
    withAuthError(to(flow?.intent === "link" ? flow.returnTo : "/"), code);

  if (query.has("error")) return fail(`auth.${provider}_denied`);
  if (
    !flow ||
    flow.provider !== provider ||
    query.get("state") !== flow.state
  ) {
    return fail("auth.state_mismatch");
  }
  const code = query.get("code");
  if (!code) return fail(`auth.${provider}_failed`);

  const body = { code, code_verifier: flow.verifier };
  if (flow.intent === "link") {
    const linked = await callApi(
      provider === "discord"
        ? api.POST("/auth/discord/link", { body })
        : api.POST("/auth/google/link", { body }),
    );
    return linked.ok ? to(flow.returnTo) : fail(linked.code);
  }
  const entered = await callApi(
    provider === "discord"
      ? api.POST("/auth/discord/login", { body })
      : api.POST("/auth/google/login", { body }),
  );
  if (!entered.ok) return fail(entered.code);
  return to(
    entered.data.deletion_cancelled ? DELETION_CANCELLED : flow.returnTo,
  );
}

/** `path` com `auth_error=<código>` na query, que a página mostra (`AuthError`). */
function withAuthError(path: string, code: string) {
  const separator = path.includes("?") ? "&" : "?";
  return `${path}${separator}auth_error=${encodeURIComponent(code)}`;
}
