/**
 * Sugere um slug a partir do nome ("Quake BR — Duelos" → "quake-br-duelos"),
 * no formato que a API aceita: a-z, 0-9 e hífen simples, até 32 caracteres.
 */
export function slugify(name: string) {
  return name
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+/, "")
    .slice(0, 32)
    .replace(/-+$/, "");
}
