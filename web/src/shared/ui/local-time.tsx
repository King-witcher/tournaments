"use client";

import { useLocale } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";

/**
 * Só a hora e o minuto de `value` (ISO), no fuso de quem vê: para prazos do
 * mesmo dia. No servidor e até hidratar sai em UTC.
 */
export function LocalTime({ value }: { value: string }) {
  const locale = useLocale();
  const isClient = useIsClient();
  const text = new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    minute: "2-digit",
    timeZone: isClient ? undefined : "UTC",
  }).format(new Date(value));
  return <time dateTime={value}>{text}</time>;
}
