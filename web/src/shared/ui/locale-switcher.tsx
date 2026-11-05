"use client";

import { useLocale, useTranslations } from "next-intl";
import { Suspense } from "react";

import { usePathname } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { Link } from "./link";
import styles from "./locale-switcher.module.scss";

function LocaleLinks({ pathname }: { pathname: string }) {
  const t = useTranslations("LocaleSwitcher");
  const current = useLocale();

  return (
    <nav aria-label={t("label")}>
      <ul className={styles.list}>
        {routing.locales.map((locale) => (
          <li key={locale} lang={locale}>
            {locale === current ? (
              <span className={styles.current} aria-current="true">
                {t(`names.${locale}`)}
              </span>
            ) : (
              <Link href={pathname} locale={locale} size="sm">
                {t(`names.${locale}`)}
              </Link>
            )}
          </li>
        ))}
      </ul>
    </nav>
  );
}

function CurrentPageLinks() {
  const pathname = usePathname();
  // O início de quem entrou é `/me` no servidor e `/` no navegador (`proxy.ts`).
  return <LocaleLinks pathname={pathname === "/me" ? "/" : pathname} />;
}

/**
 * O idioma, no rodapé (`AppFooter`): o navegador já escolhe na primeira
 * visita, então quase ninguém troca, mas quem abre um link em outro idioma
 * precisa achar o seu, com ou sem conta. Um link por idioma, para a mesma
 * página, cada um no próprio nome ("Español" também para quem lê em
 * português). O `usePathname` espera a requisição nas rotas com parâmetro;
 * até lá, os links levam ao início, no mesmo lugar.
 */
export function LocaleSwitcher() {
  return (
    <Suspense fallback={<LocaleLinks pathname="/" />}>
      <CurrentPageLinks />
    </Suspense>
  );
}
