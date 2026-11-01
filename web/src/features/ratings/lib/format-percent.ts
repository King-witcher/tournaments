/** Parte de `total` que `part` é, em porcentagem inteira ("67%"). */
export function formatPercent(part: number, total: number, locale: string) {
  return new Intl.NumberFormat(locale, {
    style: "percent",
    maximumFractionDigits: 0,
  }).format(total === 0 ? 0 : part / total);
}
