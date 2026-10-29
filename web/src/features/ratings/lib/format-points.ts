/** Pontos ganhos ou perdidos com o sinal na frente ("+29", "-29", "0"). */
export function formatPoints(points: number, locale: string) {
  return new Intl.NumberFormat(locale, { signDisplay: "exceptZero" }).format(
    points,
  );
}
