import type { TournamentValues } from "./form-values";

/**
 * A comparação do `memo` de um bloco do formulário de torneio. O bloco recebe
 * `values` inteiro, mas o tipo da prop é só o `Pick` de `keys`, então ele só
 * lê esses campos; renderiza de novo quando um deles ou outra prop muda. Sem
 * isto, cada tecla no nome renderizaria o formulário todo, inclusive o que
 * está nos grupos fechados.
 */
export function sameValues<K extends keyof TournamentValues>(
  keys: readonly K[],
) {
  return <P extends { values: Pick<TournamentValues, K> }>(
    prev: P,
    next: P,
  ) => {
    const names = Object.keys(next) as (keyof P)[];
    if (names.length !== Object.keys(prev).length) return false;
    for (const name of names) {
      if (name !== "values" && !Object.is(prev[name], next[name])) return false;
    }
    return keys.every((k) => Object.is(prev.values[k], next.values[k]));
  };
}
