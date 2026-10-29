/**
 * O rating de um time para o seeding: a média dos ratings dos jogadores,
 * arredondada (no duelo, o do jogador). Quem ainda não tem rating conta com
 * o inicial da comunidade; sem ninguém com rating, `undefined` ("Sem
 * rating").
 */
export function teamRating(
  members: { user_id: number }[],
  ratings: ReadonlyMap<number, number>,
  initial: number,
): number | undefined {
  if (!members.some((m) => ratings.has(m.user_id))) return undefined;
  const sum = members.reduce(
    (total, m) => total + (ratings.get(m.user_id) ?? initial),
    0,
  );
  return Math.round(sum / members.length);
}
