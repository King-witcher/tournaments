"use client";

import { useLocale } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";

const OPTIONS: Intl.DateTimeFormatOptions = {
  weekday: "short",
  day: "numeric",
  month: "short",
  hour: "2-digit",
  minute: "2-digit",
  timeZoneName: "short",
};

/**
 * Data e hora de `value` (ISO) no fuso e no idioma de quem vê, com o fuso
 * escrito. No servidor e até hidratar sai em UTC, que é o mesmo instante.
 */
export function LocalDateTime({ value }: { value: string }) {
  const locale = useLocale();
  const isClient = useIsClient();
  const date = new Date(value);
  const sameYear = date.getFullYear() === new Date().getFullYear();
  const text = new Intl.DateTimeFormat(locale, {
    ...OPTIONS,
    year: sameYear ? undefined : "numeric",
    timeZone: isClient ? undefined : "UTC",
  }).format(date);
  return <time dateTime={value}>{text}</time>;
}
