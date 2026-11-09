"use client";

import { useLocale, useTranslations } from "next-intl";

import { LeaveGuard } from "@/shared/ui/leave-guard";
import type { SettingsSection } from "../lib/settings-sections";

type Props = {
  /** O endereço das Configurações, sem o idioma (`/{tenant}/{torneio}/settings`). */
  settingsPath: string;
  /** As seções com mudança não salva. */
  dirty: readonly SettingsSection[];
};

/**
 * A pergunta antes de sair das Configurações com mudança não salva em alguma
 * seção (`LeaveGuard`), com as seções que perdem o que foi mudado. Trocar de
 * seção não pergunta: o que foi mudado continua lá.
 */
export function SettingsLeaveGuard({ settingsPath, dirty }: Props) {
  const t = useTranslations("TournamentSettings");
  const locale = useLocale();
  const sections = new Intl.ListFormat(locale, { type: "conjunction" }).format(
    dirty.map((s) => t(`sections.${s}`)),
  );

  return (
    <LeaveGuard
      unsaved={dirty.length > 0}
      path={settingsPath}
      subpaths
      text={t("leave.text", { sections })}
    />
  );
}
