// Data sem hora nem fuso ("AAAA-MM-DD"), como a guardam os campos de data, e
// o "dd/mm/aaaa" que eles mostram (pt e es escrevem o dia primeiro).

const pad = (n: number) => String(n).padStart(2, "0");

/** O formato do flatpickr (o calendário do `DatePicker`) para "dd/mm/aaaa". */
export const DAY_FIRST_FORMAT = "d/m/Y";

/** O que o campo aceita ao digitar: dia e mês com um ou dois dígitos. */
export const DAY_FIRST_PATTERN = "\\d{1,2}\\/\\d{1,2}\\/\\d{4}";
export const DAY_FIRST_REGEX = new RegExp(`^${DAY_FIRST_PATTERN}$`);

/** "AAAA-MM-DD" do dia que a data marca no fuso do navegador (o que o calendário escolheu). */
export function plainDateOf(date: Date) {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** "AAAA-MM-DD" → o `Date` desse dia, à meia-noite no fuso do navegador. */
export function dateOfPlain(plain: string) {
  const [year, month, day] = plain.split("-").map(Number);
  return new Date(year, month - 1, day);
}

/** "AAAA-MM-DD" → "dd/mm/aaaa". */
export function toDayFirst(plain: string) {
  const [year, month, day] = plain.split("-");
  return `${day}/${month}/${year}`;
}

/** "dd/mm/aaaa" → "AAAA-MM-DD"; `null` se não for um dia que existe. */
export function fromDayFirst(text: string) {
  const match = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(text.trim());
  if (!match) return null;
  const [day, month, year] = match.slice(1).map(Number);
  const date = new Date(year, month - 1, day);
  if (date.getMonth() !== month - 1 || date.getDate() !== day) return null;
  return plainDateOf(date);
}

/**
 * O `parseDate` do flatpickr para "dd/mm/aaaa", como o que o Carbon traz para
 * "mm/dd/aaaa": dia ou mês fora da faixa vira 1, em vez da data estranha que o
 * flatpickr montaria (32/01 em 01/02).
 */
export function parseDayFirst(text: string): Date | false {
  const [d, m, y] = text.split("/").map(Number);
  if (!y) return false;
  const month = m >= 1 && m <= 12 ? m : 1;
  const daysInMonth = new Date(y, month, 0).getDate();
  const day = d >= 1 && d <= daysInMonth ? d : 1;
  return new Date(y, month - 1, day);
}
