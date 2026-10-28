"use client";

import { type ReactNode, useLayoutEffect, useRef, useState } from "react";

import type { Edge } from "../lib/bracket-layout";
import styles from "./bracket-lines.module.scss";

type Props = {
  edges: Edge[];
  /** As colunas da chave, com `data-match` em cada partida e `data-side` em cada lado. */
  children: ReactNode;
};

/**
 * Uma curva por linha, da borda direita da partida de origem até o lado que
 * ela ocupa na seguinte: mostra para onde vai quem vence sem depender do
 * alinhamento das colunas. Mede os cartões no navegador e redesenha quando
 * algum muda de tamanho (fonte, celular, aba que aparece). Decorativa: o
 * texto do lado ("Vencedor de C1.2") já diz o mesmo.
 */
export function BracketLines({ edges, children }: Props) {
  const box = useRef<HTMLDivElement>(null);
  const [path, setPath] = useState("");

  useLayoutEffect(() => {
    const root = box.current;
    if (!root) return;
    const draw = () => {
      // Na aba escondida tudo mede zero; ela redesenha ao aparecer.
      if (root.offsetWidth === 0) return;
      const origin = root.getBoundingClientRect();
      const match = (key: string) =>
        root.querySelector<HTMLElement>(`[data-match="${key}"]`);
      setPath(
        edges
          .map(({ from, to, side }) => {
            const source = match(from);
            const target = match(to);
            if (!source || !target) return "";
            const slot =
              target.querySelectorAll<HTMLElement>("[data-side]")[
                side === "a" ? 0 : 1
              ] ?? target;
            const a = source.getBoundingClientRect();
            const b = slot.getBoundingClientRect();
            const x1 = a.right - origin.left;
            const y1 = a.top + a.height / 2 - origin.top;
            const x2 = b.left - origin.left;
            const y2 = b.top + b.height / 2 - origin.top;
            const mid = (x1 + x2) / 2;
            return `M${x1} ${y1}C${mid} ${y1} ${mid} ${y2} ${x2} ${y2}`;
          })
          .join(""),
      );
    };
    const observer = new ResizeObserver(draw);
    observer.observe(root);
    for (const card of root.querySelectorAll("[data-match]")) {
      observer.observe(card);
    }
    return () => observer.disconnect();
  }, [edges]);

  return (
    <div ref={box} className={styles.box}>
      <svg aria-hidden className={styles.lines}>
        <path d={path} />
      </svg>
      {children}
    </div>
  );
}
