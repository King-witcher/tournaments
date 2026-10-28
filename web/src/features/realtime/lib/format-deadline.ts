/**
 * Um prazo (ISO) no fuso de quem vê: só a hora se for hoje, com o dia se não
 * for. O service worker (`features/push/lib/service-worker-script.ts`) faz o
 * mesmo no texto da notificação.
 */
export function formatDeadline(
  value: string,
  locale: string,
  now = new Date(),
) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const today = date.toDateString() === now.toDateString();
  return new Intl.DateTimeFormat(
    locale,
    today
      ? { hour: "2-digit", minute: "2-digit" }
      : { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" },
  ).format(date);
}
