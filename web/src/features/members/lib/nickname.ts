/**
 * O apelido como aparece na tela. O apelido vazio é o marcador que a API
 * devolve para a conta excluída (anonimizada): no lugar dele vai `deleted`,
 * o texto traduzido de `Member.deletedAccount`.
 */
export function shownNickname(nickname: string, deleted: string) {
  return nickname === "" ? deleted : nickname;
}
