/**
 * Frags de um lado vão de -999 a 999 (a API confere de novo): no Quake
 * Champions, suicídio e queda tiram frag, então o placar de um lado pode
 * ficar negativo.
 */
export const MIN_FRAGS = -999;
export const MAX_FRAGS = 999;

/** Negativo com o sinal de menos (−), que não se confunde com o traço do placar. */
export function fragsText(n: number) {
  return n < 0 ? `−${-n}` : String(n);
}

/** Os dois lados preenchidos e iguais: não há vencedor. */
export function isTied(a: number | null, b: number | null) {
  return a !== null && b !== null && a === b;
}

/**
 * O placar que se pode enviar: os dois lados preenchidos e na faixa, sem
 * empate (o vencedor é quem fez mais). Nulo enquanto faltar algo.
 */
export function validScore(a: number | null, b: number | null) {
  const inRange = (n: number | null): n is number =>
    n !== null && n >= MIN_FRAGS && n <= MAX_FRAGS;
  return inRange(a) && inRange(b) && !isTied(a, b) ? { a, b } : null;
}
