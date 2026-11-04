"use client";

import { useLocale, useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { dateText } from "@/shared/lib/plain-date";

type Parts = { year: number; month: number; day: number; hour: number };

/** Ano, mês, dia e hora de `date` no fuso de quem vê, ou em UTC. */
function partsOf(date: Date, utc: boolean): Parts & { minute: number } {
  return utc
    ? {
        year: date.getUTCFullYear(),
        month: date.getUTCMonth() + 1,
        day: date.getUTCDate(),
        hour: date.getUTCHours(),
        minute: date.getUTCMinutes(),
      }
    : {
        year: date.getFullYear(),
        month: date.getMonth() + 1,
        day: date.getDate(),
        hour: date.getHours(),
        minute: date.getMinutes(),
      };
}

/** Dias de calendário de `from` até `to` (o horário de verão não conta). */
function daysBetween(from: Parts, to: Parts) {
  const day = (p: Parts) => Date.UTC(p.year, p.month - 1, p.day);
  return Math.round((day(to) - day(from)) / 86_400_000);
}

const RELATIVE = { [-1]: "yesterday", 0: "today", 1: "tomorrow" } as const;

type Props = {
  value: string;
  /**
   * Um instante que a frase anterior já situou: no mesmo dia que ele, sai só
   * a hora ("às 20h30"), sem repetir o dia.
   */
  sameDayAs?: string | null;
};

/**
 * Dia e hora de `value` (ISO) para o meio de uma frase, no fuso de quem vê e
 * já com a preposição: "hoje às 21h", "amanhã às 20h30", "no dia 5/11 às
 * 20h" (em espanhol, "hoy a las 21:00", "el 5/11 a las 20:00"; em inglês,
 * "today at 9 PM", "on 11/5 at 8:30 PM"). A frase em
 * volta não põe preposição nenhuma ("O torneio começa <when></when>."). Para
 * a data solta, com o fuso escrito, `LocalDateTime`. No servidor e até
 * hidratar sai em UTC, que é o mesmo instante.
 */
export function LocalDayTime({ value, sameDayAs }: Props) {
  const t = useTranslations("Common.dayTime");
  const { monthFirst } = dateText(useLocale());
  const utc = !useIsClient();
  const at = partsOf(new Date(value), utc);
  const now = partsOf(new Date(), utc);
  const known = sameDayAs ? partsOf(new Date(sameDayAs), utc) : null;
  // Relógio de 12 horas, para os idiomas que o escrevem assim (en).
  const clock = {
    hour: at.hour,
    hour12: at.hour % 12 || 12,
    period: at.hour < 12 ? "AM" : "PM",
  };
  const time =
    at.minute === 0
      ? t("hour", clock)
      : t("hourMinute", {
          ...clock,
          minute: String(at.minute).padStart(2, "0"),
        });
  const relative =
    known && daysBetween(known, at) === 0
      ? "at"
      : RELATIVE[daysBetween(now, at) as keyof typeof RELATIVE];
  const [first, second] = monthFirst ? [at.month, at.day] : [at.day, at.month];
  const date =
    at.year === now.year
      ? `${first}/${second}`
      : `${first}/${second}/${at.year}`;
  return (
    // O servidor e quem vê podem estar em dias diferentes: o navegador refaz.
    <time dateTime={value} suppressHydrationWarning>
      {t(relative ?? "date", { time, date, hour: at.hour })}
    </time>
  );
}
