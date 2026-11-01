/**
 * O último pool de mapas que a pessoa escolheu em cada comunidade, neste
 * navegador: o torneio novo dela começa com ele em vez do padrão. Fica no
 * `localStorage`; sem armazenamento (janela privada), começa do padrão.
 */

const key = (tenant: string) => `map-pool:${tenant}`;

/** O pool lembrado, só com os mapas de `active` (um mapa pode ter saído). */
export function rememberedPool(
  tenant: string,
  active: string[],
): string[] | null {
  try {
    const saved: unknown = JSON.parse(
      localStorage.getItem(key(tenant)) ?? "null",
    );
    if (!Array.isArray(saved)) return null;
    const pool = saved.filter((s) => active.includes(s));
    return pool.length > 0 ? pool : null;
  } catch {
    return null;
  }
}

export function rememberPool(tenant: string, pool: string[]) {
  try {
    localStorage.setItem(key(tenant), JSON.stringify(pool));
  } catch {
    // Vale só para este formulário.
  }
}

/** Volta ao padrão: o próximo torneio novo começa do pool padrão de agora. */
export function forgetPool(tenant: string) {
  try {
    localStorage.removeItem(key(tenant));
  } catch {
    // Nada guardado.
  }
}
