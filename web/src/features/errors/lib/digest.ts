/**
 * O `digest` que o Next põe no erro vindo do servidor (casa com o log de lá).
 * O que chega ao `error.tsx` é `unknown`: um valor lançado no cliente pode
 * nem ser um `Error`.
 */
export function digestOf(error: unknown): string | undefined {
  if (typeof error !== "object" || error === null) return undefined;
  const { digest } = error as { digest?: unknown };
  return typeof digest === "string" && digest !== "" ? digest : undefined;
}
