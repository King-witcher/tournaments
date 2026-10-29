/**
 * Caminho interno (sem idioma) para voltar depois do login, vindo de
 * `?return_to=` (ou do fluxo do OAuth guardado na aba). Qualquer outra coisa
 * vira "/": nada de voltar para fora do site.
 */
export function safeReturnTo(raw: string | string[] | undefined) {
  if (typeof raw !== "string") return "/";
  if (!raw.startsWith("/") || raw.startsWith("//") || raw.startsWith("/\\")) {
    return "/";
  }
  return raw;
}
