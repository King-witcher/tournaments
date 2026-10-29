/**
 * Como a lista mostra um link de convite: o começo do endereço, só para
 * reconhecê-lo (`…/invite/7Fq2…`). Nulo nos links de antes de a API guardar
 * o começo do token.
 */
export function inviteHint(hint: string | null | undefined) {
  return hint ? `…/invite/${hint}…` : null;
}
