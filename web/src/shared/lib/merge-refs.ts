import type { Ref, RefCallback } from "react";

/**
 * Um ref que repassa o elemento a vários (o do componente e o de quem o
 * usa). Crie com `useMemo` sobre os refs, para o React não soltar e prender
 * o elemento a cada render.
 */
export function mergeRefs<T>(...refs: (Ref<T> | undefined)[]): RefCallback<T> {
  return (value) => {
    for (const ref of refs) {
      if (typeof ref === "function") ref(value);
      else if (ref) ref.current = value;
    }
  };
}
