import { safeReturnTo } from "./return-to";

const KEY = "signup-return-to";

/** O link do cadastro vale 1 hora; um destino mais velho que isso não vale. */
const TTL_MS = 60 * 60 * 1000;

/**
 * Guarda, neste navegador, para onde o cadastro voltaria (`?return_to=`, o
 * convite que trouxe a pessoa): a conta só nasce quando o link do email é
 * aberto, numa página que não tem o `return_to`. Aberto no mesmo navegador,
 * o link leva para lá; em outro (o app de email do celular), ou sem
 * armazenamento (aba anônima, bloqueado), vai para o início.
 */
export function rememberSignupReturnTo(returnTo: string) {
  try {
    localStorage.setItem(
      KEY,
      JSON.stringify({ path: returnTo, at: Date.now() }),
    );
  } catch {
    // Sem armazenamento: depois de confirmar, vai para o início.
  }
}

/** O destino guardado por `rememberSignupReturnTo` (e o apaga), ou "/". */
export function takeSignupReturnTo() {
  try {
    const raw = localStorage.getItem(KEY);
    localStorage.removeItem(KEY);
    if (!raw) return "/";
    const { path, at } = JSON.parse(raw) as { path?: unknown; at?: unknown };
    if (typeof at !== "number" || Date.now() - at > TTL_MS) return "/";
    return safeReturnTo(typeof path === "string" ? path : undefined);
  } catch {
    return "/";
  }
}
