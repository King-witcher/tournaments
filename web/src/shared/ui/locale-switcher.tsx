"use client";

import { Translate } from "@carbon/icons-react";
import { Switcher, SwitcherItem } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";

import { Link as IntlLink, usePathname } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { HeaderPanelAction } from "./header-panel-action";

/**
 * Troca o idioma mantendo a página: ação com o ícone `Translate` no
 * cabeçalho, abrindo um painel com os idiomas. Cada idioma aparece no
 * próprio nome ("Español" também para quem lê em português), como se espera
 * de um seletor.
 */
export function LocaleSwitcher() {
  const t = useTranslations("LocaleSwitcher");
  const current = useLocale();
  const pathname = usePathname();

  return (
    <HeaderPanelAction label={t("label")} icon={<Translate size={20} />}>
      <Switcher aria-label={t("label")} expanded>
        {routing.locales.map((locale) => (
          <SwitcherItem
            key={locale}
            // O SwitcherItem repassa `as` e `locale` ao Link do i18n, mas o
            // tipo dele não os declara.
            {...({ as: IntlLink, locale, lang: locale } as object)}
            href={pathname}
            isSelected={locale === current}
            aria-current={locale === current ? "true" : undefined}
            aria-label={t(`names.${locale}`)}
          >
            {t(`names.${locale}`)}
          </SwitcherItem>
        ))}
      </Switcher>
    </HeaderPanelAction>
  );
}
