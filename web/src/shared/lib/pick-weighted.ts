/** Uma opção de sorteio: `weight` é relativo aos das outras (0 nunca sai). */
export type Weighted<T> = { value: T; weight: number };

/** Sorteia uma opção pelo peso; sem opção com peso, `undefined`. */
export function pickWeighted<T>(
  options: readonly Weighted<T>[],
  random: () => number = Math.random,
): T | undefined {
  const total = options.reduce((sum, o) => sum + Math.max(0, o.weight), 0);
  if (total <= 0) return undefined;
  let left = random() * total;
  for (const option of options) {
    left -= Math.max(0, option.weight);
    if (left < 0) return option.value;
  }
  return options.findLast((o) => o.weight > 0)?.value;
}
