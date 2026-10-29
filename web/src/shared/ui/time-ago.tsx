"use client";

import { useLocale } from "next-intl";
import { useEffect, useState } from "react";

/** Do maior para o menor: a primeira que cabe no intervalo. */
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60],
];

/** De quanto em quanto tempo o texto se refaz. */
const EVERY_MS = 30_000;

/**
 * Há quanto tempo foi `value` (ISO), no idioma de quem vê ("há 5 minutos",
 * "agora", "ontem"), refeito a cada 30 s: a aba fica aberta durante o
 * torneio. Para o horário exato, `LocalDateTime`. O servidor escreve com o
 * relógio dele, que difere do de quem vê por segundos: a hidratação não
 * confere o texto, e o navegador o refaz.
 */
export function TimeAgo({ value }: { value: string }) {
  const locale = useLocale();
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    setNow(Date.now());
    const id = window.setInterval(() => setNow(Date.now()), EVERY_MS);
    return () => window.clearInterval(id);
  }, []);

  const seconds = Math.max(0, (now - new Date(value).getTime()) / 1000);
  const unit = UNITS.find(([, size]) => seconds >= size);
  const format = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  const text = unit
    ? format.format(-Math.floor(seconds / unit[1]), unit[0])
    : format.format(0, "second");
  return (
    <time dateTime={value} suppressHydrationWarning>
      {text}
    </time>
  );
}
