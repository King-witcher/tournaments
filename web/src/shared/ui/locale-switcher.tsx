"use client";

import { useLocale, useTranslations } from "next-intl";
import { useId } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import { routing } from "@/i18n/routing";
import { ContentSwitcher } from "./content-switcher";
import styles from "./locale-switcher.module.scss";

type Locale = (typeof routing.locales)[number];

/**
 * O idioma, no painel da pessoa (`UserPanel`), embaixo do tema e do mesmo
 * jeito: um `ContentSwitcher`, cujo valor é o idioma atual (o Carbon não
 * marca item escolhido num painel do cabeçalho, mas um controle mostra o
 * valor). Escolher outro abre a mesma página nele. As setas só mudam o foco
 * e o Enter escolhe, para não trocar de página a cada seta. Cada idioma
 * aparece no próprio nome ("Español" também para quem lê em português),
 * como se espera de um seletor.
 */
export function LocaleSwitcher() {
  const t = useTranslations("LocaleSwitcher");
  const current = useLocale() as Locale;
  const pathname = usePathname();
  const router = useRouter();
  const labelId = useId();

  return (
    <div className={styles.locale}>
      <p id={labelId} className={styles.label}>
        {t("label")}
      </p>
      <ContentSwitcher
        aria-labelledby={labelId}
        selectionMode="manual"
        options={routing.locales.map((locale) => ({
          value: locale,
          text: t(`names.${locale}`),
          lang: locale,
        }))}
        value={current}
        onChange={(locale) => router.replace(pathname, { locale })}
      />
    </div>
  );
}
