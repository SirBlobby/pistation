let sequence = 0;

export function createId(prefix: string): string {
  sequence += 1;
  const random = Math.random().toString(36).slice(2, 10);
  return `${prefix}-${Date.now().toString(36)}-${sequence.toString(36)}-${random}`;
}

export function ensureUniqueIds<T>(
  items: T[],
  getId: (item: T) => string,
  withId: (item: T, id: string) => T,
  prefix: string
): T[] {
  const seen = new Set<string>();

  return items.map((item) => {
    const id = getId(item);
    if (id && !seen.has(id)) {
      seen.add(id);
      return item;
    }

    const replacement = createId(prefix);
    seen.add(replacement);
    return withId(item, replacement);
  });
}
