// Data sem hora nem fuso ("AAAA-MM-DD"), como a guardam os campos de data, e
// o texto que eles mostram: "dd/mm/aaaa" em pt e es (dia primeiro) e
// "mm/dd/yyyy" em en.

import { dateDigits } from "./input-filters";

const pad = (n: number) => String(n).padStart(2, "0");

/** "AAAA-MM-DD" do dia que a data marca no fuso do navegador (o que o calendário escolheu). */
export function plainDateOf(date: Date) {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** "AAAA-MM-DD" → o `Date` desse dia, à meia-noite no fuso do navegador. */
export function dateOfPlain(plain: string) {
  const [year, month, day] = plain.split("-").map(Number);
  return new Date(year, month - 1, day);
}

/** Como o campo de data escreve, lê e filtra o texto de uma data. */
export type DateText = {
  /** O mês vem antes do dia ("mm/dd/yyyy"). */
  monthFirst: boolean;
  /** O formato do flatpickr (o calendário do `DatePicker`). */
  format: string;
  /** O que o campo aceita ao digitar: dia e mês com um ou dois dígitos. */
  pattern: string;
  regex: RegExp;
  /** O que se digita: só dígitos, com as barras postas sozinhas. */
  digits: (value: string) => string;
  /** "AAAA-MM-DD" → o texto do campo. */
  toText: (plain: string) => string;
  /** O texto do campo → "AAAA-MM-DD"; `null` se não for um dia que existe. */
  fromText: (text: string) => string | null;
  /**
   * O `parseDate` do flatpickr, como o que o Carbon traz para "mm/dd/aaaa":
   * dia ou mês fora da faixa vira 1, em vez da data estranha que o flatpickr
   * montaria (32/01 em 01/02).
   */
  parse: (text: string) => Date | false;
};

const PATTERN = "\\d{1,2}\\/\\d{1,2}\\/\\d{4}";
const REGEX = new RegExp(`^${PATTERN}$`);

function build(monthFirst: boolean): DateText {
  // Os dois primeiros números do texto, na ordem em que o idioma os escreve.
  const parts = (text: string) => {
    const [a, b, year] = text.split("/").map(Number);
    return monthFirst ? { year, month: a, day: b } : { year, month: b, day: a };
  };
  return {
    monthFirst,
    format: monthFirst ? "m/d/Y" : "d/m/Y",
    pattern: PATTERN,
    regex: REGEX,
    digits: (value) => dateDigits(value, monthFirst),
    toText(plain) {
      const [year, month, day] = plain.split("-");
      return monthFirst ? `${month}/${day}/${year}` : `${day}/${month}/${year}`;
    },
    fromText(text) {
      const trimmed = text.trim();
      if (!REGEX.test(trimmed)) return null;
      const { year, month, day } = parts(trimmed);
      const date = new Date(year, month - 1, day);
      if (date.getMonth() !== month - 1 || date.getDate() !== day) return null;
      return plainDateOf(date);
    },
    parse(text) {
      const { year, month: m, day: d } = parts(text);
      if (!year) return false;
      const month = m >= 1 && m <= 12 ? m : 1;
      const daysInMonth = new Date(year, month, 0).getDate();
      const day = d >= 1 && d <= daysInMonth ? d : 1;
      return new Date(year, month - 1, day);
    },
  };
}

const DAY_FIRST = build(false);
const MONTH_FIRST = build(true);

/** O texto de data de `locale`: mês primeiro em `en`, dia primeiro nos outros. */
export function dateText(locale: string): DateText {
  return locale === "en" ? MONTH_FIRST : DAY_FIRST;
}
