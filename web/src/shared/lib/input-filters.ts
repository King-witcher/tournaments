/**
 * Filtros para a prop `filter` dos campos de texto: tiram o que o campo não
 * aceita enquanto a pessoa digita ou cola. Devolvem o texto como deve ficar.
 */

/** Só dígitos. */
export function digitsOnly(value: string) {
  return value.replace(/\D/g, "");
}

/** Identificador na URL (slug): a-z, 0-9 e hífen simples, sem hífen no começo. */
export function slugChars(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, "")
    .replace(/-{2,}/g, "-")
    .replace(/^-/, "");
}
