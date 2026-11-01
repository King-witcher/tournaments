// Data e hora "de parede" (o que se lê no relógio de um fuso) e o instante em
// UTC que elas dão. Os campos de data e hora trabalham com a data em
// "AAAA-MM-DD" e a hora em "HH:mm"; a API, com ISO em UTC.

const pad = (n: number) => String(n).padStart(2, "0");

/** Fuso do navegador (ex.: "America/Sao_Paulo"). Só no cliente. */
export function browserTimeZone() {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}

const formatters = new Map<string, Intl.DateTimeFormat>();

function partsIn(timeZone: string, instant: number) {
  let format = formatters.get(timeZone);
  if (!format) {
    format = new Intl.DateTimeFormat("en-US", {
      timeZone,
      hourCycle: "h23",
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
    formatters.set(timeZone, format);
  }
  const parts = format.formatToParts(instant);
  const get = (type: Intl.DateTimeFormatPartTypes) =>
    Number(parts.find((p) => p.type === type)?.value);
  return {
    year: get("year"),
    month: get("month"),
    day: get("day"),
    hour: get("hour"),
    minute: get("minute"),
    second: get("second"),
  };
}

/** Quanto o fuso está à frente do UTC nesse instante, em ms (−3 h dá negativo). */
function offsetAt(timeZone: string, instant: number) {
  const p = partsIn(timeZone, instant);
  const wall = Date.UTC(p.year, p.month - 1, p.day, p.hour, p.minute, p.second);
  return wall - Math.floor(instant / 1000) * 1000;
}

/** A data ("AAAA-MM-DD") e a hora ("HH:mm") que o instante ISO mostra no fuso. */
export function toZoned(iso: string, timeZone: string) {
  const p = partsIn(timeZone, new Date(iso).getTime());
  return {
    date: `${p.year}-${pad(p.month)}-${pad(p.day)}`,
    time: `${pad(p.hour)}:${pad(p.minute)}`,
  };
}

const DATE = /^(\d{4})-(\d{2})-(\d{2})$/;
const TIME = /^([01]?\d|2[0-3]):([0-5]\d)$/;

/** A hora digitada é uma hora de 24 h ("9:30", "21:05")? */
export function isTime(time: string) {
  return TIME.test(time);
}

/**
 * O instante (ISO em UTC) da data e hora lidas no fuso; data ou hora vazia
 * ou inválida dá `null`. Na hora que não existe (o relógio adiantou no
 * horário de verão), vale a mesma hora com o deslocamento de antes.
 */
export function fromZoned(date: string, time: string, timeZone: string) {
  const d = DATE.exec(date);
  const t = TIME.exec(time);
  if (!d || !t) return null;
  const wall = Date.UTC(
    Number(d[1]),
    Number(d[2]) - 1,
    Number(d[3]),
    Number(t[1]),
    Number(t[2]),
  );
  // O deslocamento depende do instante, que depende do deslocamento: duas
  // voltas resolvem até a troca do horário de verão.
  const first = wall - offsetAt(timeZone, wall);
  const second = wall - offsetAt(timeZone, first);
  return new Date(second).toISOString();
}

let zones: { value: string; label: string }[] | undefined;

/**
 * Os fusos do navegador para escolher, pela cidade em ordem alfabética, com
 * o deslocamento de agora ("Sao Paulo (GMT-3)"): a cidade basta para achar e
 * cabe ao lado da hora no celular. Ficam de fora os "Etc/GMT+3" (o sinal é
 * o contrário do que parece); o do navegador entra mesmo quando a lista o
 * chama por outro nome. Só no cliente.
 */
export function timeZoneOptions() {
  if (!zones) {
    const own = browserTimeZone();
    const names = Intl.supportedValuesOf("timeZone").filter(
      (name) => !name.startsWith("Etc/"),
    );
    if (!names.includes(own)) names.push(own);
    if (!names.includes("UTC")) names.push("UTC");
    const now = Date.now();
    zones = names
      .map((value) => {
        const offset = new Intl.DateTimeFormat("en-US", {
          timeZone: value,
          timeZoneName: "shortOffset",
        })
          .formatToParts(now)
          .find((p) => p.type === "timeZoneName")?.value;
        const city = value.split("/").at(-1)?.replaceAll("_", " ");
        return { value, label: `${city} (${offset})` };
      })
      .sort((a, b) => a.label.localeCompare(b.label));
  }
  return zones;
}
