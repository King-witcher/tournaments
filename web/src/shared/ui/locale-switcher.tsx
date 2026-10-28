"use client";

import { useLocale, useTranslations } from "next-intl";

import { Link, usePathname } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";

/**
 * Troca o idioma mantendo a página. Cada idioma aparece no próprio nome
 * ("Español" também para quem lê em português), como se espera de um seletor.
 */
export function LocaleSwitcher() {
  const t = useTranslations("LocaleSwitcher");
  const current = useLocale();
  const pathname = usePathname();

  return (
    <nav aria-label={t("label")} className="flex gap-2 text-sm">
      {routing.locales.map((locale) =>
        locale === current ? (
          <span
            key={locale}
            aria-current="true"
            className="inline-flex min-h-11 items-center sm:min-h-0 font-medium"
          >
            {t(`names.${locale}`)}
          </span>
        ) : (
          <Link
            key={locale}
            href={pathname}
            locale={locale}
            className="inline-flex min-h-11 items-center sm:min-h-0 text-muted underline underline-offset-4 hover:decoration-2"
          >
            {t(`names.${locale}`)}
          </Link>
        ),
      )}
    </nav>
  );
}
