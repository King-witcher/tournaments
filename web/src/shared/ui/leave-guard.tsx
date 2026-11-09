"use client";

import { useLocale, useTranslations } from "next-intl";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { ConfirmModal } from "./confirm-modal";

type Props = {
  /** Há mudança não salva: sair pergunta. */
  unsaved: boolean;
  /** O endereço da página, sem o idioma (`/{tenant}/settings`). */
  path: string;
  /** As páginas abaixo de `path` não contam como sair: o que foi mudado continua lá. */
  subpaths?: boolean;
  /** O texto da pergunta; padrão, "As mudanças não salvas se perdem". */
  text?: ReactNode;
};

/**
 * A pergunta antes de sair de uma página com mudança não salva. Fechar ou
 * recarregar a aba do navegador fica com o aviso dele (`beforeunload`); um
 * link do site para fora da página (o menu, a trilha, o idioma) abre o
 * `ConfirmModal`, e "Sair sem salvar" segue o link. O Voltar do navegador não
 * passa por aqui (o Next não deixa segurar a navegação).
 *
 * Exemplo: `<LeaveGuard unsaved={dirty} path="/account" />`, em volta de um
 * formulário em que a pessoa perde o que digitou ao sair.
 */
export function LeaveGuard({ unsaved, path, subpaths = false, text }: Props) {
  const t = useTranslations("Common.leave");
  const locale = useLocale();
  const [leaving, setLeaving] = useState<HTMLAnchorElement>();
  // O clique de "Sair sem salvar", que segue o link sem perguntar de novo.
  const bypass = useRef(false);

  useEffect(() => {
    if (!unsaved) return;
    const prefix = `/${locale}${path}`;
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
    };
    // Na captura da janela, antes do `Link` do Next, que navega no clique.
    const onClick = (event: MouseEvent) => {
      if (bypass.current) {
        bypass.current = false;
        return;
      }
      if (
        event.defaultPrevented ||
        event.button !== 0 ||
        event.metaKey ||
        event.ctrlKey ||
        event.shiftKey ||
        event.altKey
      ) {
        return;
      }
      const link =
        event.target instanceof Element
          ? event.target.closest<HTMLAnchorElement>("a[href]")
          : null;
      if (!link || link.hasAttribute("download")) return;
      if (link.target && link.target !== "_self") return;
      const url = new URL(link.href, window.location.href);
      // Fora do site, o navegador descarrega a página: vale o `beforeunload`.
      if (url.origin !== window.location.origin) return;
      if (
        url.pathname === prefix ||
        (subpaths && url.pathname.startsWith(`${prefix}/`))
      ) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      setLeaving(link);
    };
    window.addEventListener("beforeunload", onBeforeUnload);
    window.addEventListener("click", onClick, true);
    return () => {
      window.removeEventListener("beforeunload", onBeforeUnload);
      window.removeEventListener("click", onClick, true);
    };
  }, [unsaved, locale, path, subpaths]);

  return (
    <ConfirmModal
      open={leaving !== undefined && unsaved}
      title={t("title")}
      confirmLabel={t("confirm")}
      destructive
      onConfirm={() => {
        const link = leaving;
        setLeaving(undefined);
        bypass.current = true;
        link?.click();
      }}
      onClose={() => {
        const link = leaving;
        setLeaving(undefined);
        // O Modal sai da árvore e o foco cairia no body.
        requestAnimationFrame(() => link?.focus());
      }}
    >
      {text ?? t("text")}
    </ConfirmModal>
  );
}
