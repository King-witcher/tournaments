"use client";

import { useLocale } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";

/**
 * O dia (sem hora) de `value` no fuso e no idioma de quem vê: "membro desde".
 * No servidor e até hidratar sai em UTC, que pode ser o dia vizinho.
 */
export function MemberSince({ value }: { value: string }) {
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
