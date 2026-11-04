"use client";

import { useEffect, useRef } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";

/** A página que entra sozinha ao carregar: o cookie novo não pode sair. */
const SIGNS_IN_ON_LOAD = /^\/auth\/[^/]+\/callback$/;

/**
 * Quanto o início espera a apresentação depois da limpeza. Ela não depende da
 * API, então só passa disso se o cookie não saiu de verdade (nome ou domínio
 * errados no deploy): o refresh trouxe este componente de novo.
 */
const HOME_FALLBACK_MS = 8000;

/**
 * Para o cookie que a API recusou: só ela o apaga de verdade (ele é `HttpOnly`,
 * com o domínio dela). Não é o logout, que encerraria a sessão nova se outra aba
 * entrou nesse meio tempo: esta rota só tira o cookie de sessão que não vale.
 * Uma vez por página aberta, sem laço: sem o cookie, o proxy serve a apresentação
 * em `/`, e este componente sai da árvore; nas outras páginas só falta o refresh.
 * Se a limpeza falhar ou o cookie ficar, o início (que esperaria a sessão para
 * sempre) vai para o login, que não redireciona; nas outras, a pessoa continua
 * como anônima, e a próxima página tenta de novo. Não desenha nada.
 */
export function ClearExpiredSession() {
  const router = useRouter();
  const pathname = usePathname();
  // Uma vez só, também no efeito dobrado do modo estrito.
  const started = useRef(false);
  const fallback = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => () => clearTimeout(fallback.current), []);

  useEffect(() => {
    if (started.current || SIGNS_IN_ON_LOAD.test(pathname)) return;
    started.current = true;
    // `/me` é o caminho reescrito de `/` (o proxy o serve lá).
    const atHome = pathname === "/" || pathname === "/me";
    const toLogin = () => router.replace("/login");
    api.POST("/auth/sessions/clear-expired").then(
      ({ response }) => leave(response.ok),
      () => leave(false),
    );

    function leave(cleared: boolean) {
      if (cleared) {
        if (atHome) {
          router.replace("/");
          fallback.current = setTimeout(toLogin, HOME_FALLBACK_MS);
        }
        router.refresh();
      } else if (atHome) {
        toLogin();
      }
    }
  }, [pathname, router]);

  return null;
}
