import type { Metadata, ResolvingMetadata } from "next";
import type { Locale } from "next-intl";
import { getLocale, getTranslations } from "next-intl/server";

import { routing } from "@/i18n/routing";

const productionHost = process.env.VERCEL_PROJECT_PRODUCTION_URL;

/**
 * A base das URLs relativas das tags (`canonical`, `og:url`): o domínio de
 * produção em toda implantação, inclusive no staging, que não deve concorrer
 * com ele em busca nenhuma.
 */
export const SITE_ORIGIN = new URL(
  productionHost ? `https://${productionHost}` : "http://localhost:4001",
);

/** O idioma do site no formato do `og:locale`. */
const OG_LOCALES: Record<Locale, string> = {
  pt: "pt_BR",
  es: "es_LA",
};

/**
 * O que toda prévia de link tem em Open Graph. Quem define `openGraph` troca o
 * do layout inteiro, então parte daqui e acrescenta título, descrição e imagem.
 */
export async function siteOpenGraph(locale?: Locale) {
  const current = locale ?? (await getLocale());
  const t = await getTranslations({ locale: current, namespace: "Nav" });
  return {
    siteName: t("brand"),
    type: "website" as const,
    locale: OG_LOCALES[current],
    alternateLocale: routing.locales
      .filter((other) => other !== current)
      .map((other) => OG_LOCALES[other]),
  };
}

/**
 * As tags de uma página aberta a buscadores (o layout deixa as outras de fora):
 * o endereço canônico e a prévia com a imagem do site. `path` vem sem o idioma.
 */
export async function publicPageMetadata(
  path: string,
  preview: { title: string; description: string },
  parent: ResolvingMetadata,
): Promise<Metadata> {
  const locale = await getLocale();
  const url = `/${locale}${path}`;
  return {
    alternates: { canonical: url },
    robots: { index: true, follow: true },
    openGraph: {
      ...(await siteOpenGraph()),
      ...preview,
      url,
      images: (await parent).openGraph?.images,
    },
  };
}
