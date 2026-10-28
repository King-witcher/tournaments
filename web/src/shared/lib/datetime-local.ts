const pad = (n: number) => String(n).padStart(2, "0");

/**
 * Valor de `<input type="datetime-local">` ("AAAA-MM-DDTHH:mm", no fuso do
 * navegador) para um instante ISO. Só no cliente: no servidor o fuso é outro.
 */
export function toLocalInput(iso: string) {
  const d = new Date(iso);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** O valor do input, lido no fuso do navegador, como ISO em UTC; vazio ou inválido → `null`. */
export function fromLocalInput(value: string) {
  if (!value) return null;
  const d = new Date(value);
  return Number.isNaN(d.getTime()) ? null : d.toISOString();
}

/** Fuso do navegador (ex.: "America/Sao_Paulo"). Só no cliente. */
export function browserTimeZone() {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}
