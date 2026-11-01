import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { ImageResponse } from "next/og";
import { hasLocale } from "next-intl";

import { routing } from "@/i18n/routing";
import { BrandCard } from "@/shared/lib/brand-card";
import es from "../../../messages/es/Landing.json";
import pt from "../../../messages/pt/Landing.json";

export const size = { width: 1200, height: 630 };
export const contentType = "image/png";
export const alt = "Telefrag";

// Os textos da apresentação vêm direto dos JSON, como no `sw.js`: o
// `getTranslations` lê o idioma de `next/root-params`, que uma rota de imagem
// ainda não tem.
const TEXTS = {
  pt: { title: pt.hero.title, description: pt.hero.lead },
  es: { title: es.hero.title, description: es.hero.lead },
};

// A IBM Plex Sans em WOFF (o `ImageResponse` não lê WOFF2, o formato do
// next/font), do pacote que o Carbon instala.
const plex = (weight: string) =>
  readFile(
    join(
      process.cwd(),
      `node_modules/@ibm/plex-sans/fonts/complete/woff/IBMPlexSans-${weight}.woff`,
    ),
  );

/**
 * A prévia dos links do site (Discord, WhatsApp) em todas as páginas, no
 * idioma do link: o logo, o título e a frase da apresentação. Gerada no
 * build, uma por idioma.
 */
export default async function OpengraphImage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const requested = (await params).locale;
  const locale = hasLocale(routing.locales, requested)
    ? requested
    : routing.defaultLocale;
  const [regular, semibold] = await Promise.all([
    plex("Regular"),
    plex("SemiBold"),
  ]);
  return new ImageResponse(<BrandCard {...size} {...TEXTS[locale]} />, {
    ...size,
    fonts: [
      { name: "IBM Plex Sans", data: regular, weight: 400 },
      { name: "IBM Plex Sans", data: semibold, weight: 600 },
    ],
  });
}
