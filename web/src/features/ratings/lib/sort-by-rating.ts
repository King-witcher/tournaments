/**
 * A ordem por rating, para o seeding: maior rating primeiro; no empate e
 * entre quem não tem rating (estes no fim), a ordem em que já estavam.
 */
export function sortByRating<T extends { user_id: number }>(
  order: T[],
  ratings: ReadonlyMap<number, number>,
): T[] {
  return order
    .map((item, index) => ({ item, index, rating: ratings.get(item.user_id) }))
    .sort((a, b) => {
      if (a.rating !== undefined && b.rating !== undefined) {
        return b.rating - a.rating || a.index - b.index;
      }
      if (a.rating !== undefined) return -1;
      if (b.rating !== undefined) return 1;
      return a.index - b.index;
    })
    .map(({ item }) => item);
}
