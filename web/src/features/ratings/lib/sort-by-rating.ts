/**
 * A ordem por rating, para o seeding: maior rating primeiro; no empate e
 * entre quem não tem rating (estes no fim), a ordem em que já estavam.
 * `ratings` vai pelo `id` de cada item (o time, no seeding).
 */
export function sortByRating<T extends { id: number }>(
  order: T[],
  ratings: ReadonlyMap<number, number>,
): T[] {
  return order
    .map((item, index) => ({ item, index, rating: ratings.get(item.id) }))
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
