"use client";

import { useLocale, useTranslations } from "next-intl";
import { useEffect, useRef, useState } from "react";

import { ConfirmModal } from "@/shared/ui/confirm-modal";
import type { SettingsSection } from "../lib/settings-sections";

type Props = {
  /** O endereço das Configurações, sem o idioma (`/{tenant}/{torneio}/settings`). */
  settingsPath: string;
  /** As seções com mudança não salva. */
  dirty: readonly SettingsSection[];
};

/**
 * A pergunta antes de sair das Configurações com mudança não salva em alguma
 * seção. Fechar ou recarregar a aba do navegador fica com o aviso dele
 * (`beforeunload`); um link do site para fora das Configurações (o menu, a
 * trilha, o idioma) abre o `ConfirmModal`, com as seções que perdem o que foi
 * mudado, e "Sair sem salvar" segue o link. Trocar de seção não pergunta: o
 * que foi mudado continua lá. O Voltar do navegador não passa por aqui (o
 * Next não deixa segurar a navegação).
 */
export function SettingsLeaveGuard({ settingsPath, dirty }: Props) {
  const t = useTranslations("TournamentSettings");
  const locale = useLocale();
  const [leaving, setLeaving] = useState<HTMLAnchorElement>();
  // O clique de "Sair sem salvar", que segue o link sem perguntar de novo.
  const bypass = useRef(false);
  const unsaved = dirty.length > 0;

  useEffect(() => {
    if (!unsaved) return;
    const prefix = `/${locale}${settingsPath}`;
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
      if (url.pathname === prefix || url.pathname.startsWith(`${prefix}/`)) {
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
  }, [unsaved, locale, settingsPath]);

  const sections = new Intl.ListFormat(locale, { type: "conjunction" }).format(
    dirty.map((s) => t(`sections.${s}`)),
  );

  return (
    <ConfirmModal
      open={leaving !== undefined && unsaved}
      title={t("leave.title")}
      confirmLabel={t("leave.confirm")}
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
      {t("leave.text", { sections })}
    </ConfirmModal>
  );
}
