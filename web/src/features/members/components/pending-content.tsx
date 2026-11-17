"use client";

import type { ReactNode } from "react";

import { TabLoading } from "@/shared/ui/skeleton";
import { useSearchTransition } from "../hooks/use-search-transition";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
  /** Os esqueletos na forma do conteúdo. */
  fallback: ReactNode;
  children: ReactNode;
};

/** O conteúdo que um filtro troca: esqueleto enquanto a troca está no ar. */
export function PendingContent({ label, fallback, children }: Props) {
  const { pending } = useSearchTransition();
  return pending ? <TabLoading label={label}>{fallback}</TabLoading> : children;
}
