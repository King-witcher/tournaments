"use client";

import { useLocale } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";

/**
 * Só o dia de `value` (ISO), sem hora, no fuso e no idioma de quem vê: "membro
 * desde", datas de histórico. No servidor e até hidratar sai em UTC, que pode
 * ser o dia vizinho. Com hora, `LocalDateTime`.
 */
export function LocalDate({ value }: { value: string }) {
  const locale = useLocale();
  const isClient = useIsClient();
  const text = new Intl.DateTimeFormat(locale, {
    day: "numeric",
    month: "short",
    year: "numeric",
    timeZone: isClient ? undefined : "UTC",
  }).format(new Date(value));
  return <time dateTime={value}>{text}</time>;
}
