/**
 * Junta classes, ignorando as falsas. Não resolve conflito do Tailwind (não há
 * tailwind-merge): não passe duas classes para a mesma propriedade; use uma
 * variante.
 */
export function cn(...classes: (string | false | null | undefined)[]) {
  return classes.filter(Boolean).join(" ");
}
