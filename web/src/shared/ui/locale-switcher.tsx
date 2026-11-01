"use client";

import { Switcher, SwitcherItem } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import { useId } from "react";

import { Link as IntlLink, usePathname } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import styles from "./locale-switcher.module.scss";

/**
 * O idioma, no painel da pessoa (`UserPanel`), embaixo do tema: os idiomas
 * como links para a mesma página, o atual marcado. Cada idioma aparece no
 * próprio nome ("Español" também para quem lê em português), como se espera
 * de um seletor.
 */
export function LocaleSwitcher() {
  const t = useTranslations("LocaleSwitcher");
  const current = useLocale();
  const pathname = usePathname();
  const labelId = useId();

  return (
    <div className={styles.locale}>
      <p id={labelId} className={styles.label}>
        {t("label")}
      </p>
      <Switcher aria-labelledby={labelId} expanded className={styles.list}>
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
    </div>
  );
}
