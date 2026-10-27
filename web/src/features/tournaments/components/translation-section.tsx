import { useTranslations } from "next-intl";

import { routing } from "@/i18n/routing";
import type { components } from "@/shared/api/schema";
import { TranslationForm } from "./translation-form";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/** Uma tradução opcional por idioma diferente do original. */
export function TranslationSection({ tenant, tournament }: Props) {
  const t = useTranslations("Tournament.translation");
  const tLocales = useTranslations("Locales");
  const others = routing.locales.filter((l) => l !== tournament.locale);

  return others.map((locale) => (
    <section key={locale} className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">
        {t("title", { language: tLocales(locale) })}
      </h2>
      <p className="text-sm opacity-70">{t("hint")}</p>
      <TranslationForm
        tenant={tenant}
        tournament={tournament.slug}
        locale={locale}
        translation={tournament.translations.find((x) => x.locale === locale)}
      />
    </section>
  ));
}
