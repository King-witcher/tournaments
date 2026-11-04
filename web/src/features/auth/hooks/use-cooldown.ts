import { useEffect, useState } from "react";

/**
 * Conta `seconds` para baixo, de segundo em segundo: `left` chega a 0, e
 * `restart` recomeça. Começa contando.
 */
export function useCooldown(seconds: number) {
  const [left, setLeft] = useState(seconds);
  useEffect(() => {
    if (left <= 0) return;
    const timer = setTimeout(() => setLeft(left - 1), 1000);
    return () => clearTimeout(timer);
  }, [left]);
  return { left, restart: () => setLeft(seconds) };
}
