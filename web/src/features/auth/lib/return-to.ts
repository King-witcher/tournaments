/**
 * Caminho interno (sem idioma) para voltar depois do login, vindo de
 * `?return_to=`. Qualquer outra coisa vira "/", como a API faz no Discord.
 */
export function safeReturnTo(raw: string | string[] | undefined) {
  if (typeof raw !== "string") return "/";
  if (!raw.startsWith("/") || raw.startsWith("//") || raw.startsWith("/\\")) {
    return "/";
  }
  return raw;
}
