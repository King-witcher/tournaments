import { ImageResponse } from "next/og";

import { routing } from "@/i18n/routing";
import { BrandCard } from "@/shared/lib/brand-card";
import nav from "../../../messages/pt/Nav.json";

export const size = { width: 1200, height: 630 };
export const contentType = "image/png";
export const alt = nav.brand;

export function generateStaticParams() {
  return routing.locales.map((locale) => ({ locale }));
}

/**
 * A imagem da prévia dos links do site (Discord, WhatsApp), igual nos dois
 * idiomas, gerada no build. Fica neste segmento, e não na raiz de `app/`: o
 * `openGraph` do layout descartaria a imagem de um segmento acima dele. O convite
 * com capa ou logo e a sala de veto trocam por outra.
 */
export default function OpengraphImage() {
  return new ImageResponse(<BrandCard {...size} />, size);
}
