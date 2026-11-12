import type { ReactNode } from "react";

import { Tag } from "./carbon/tag";

/**
 * Cor fixa por significado (docs/design.md, "Estado"). São poucas porque a
 * etiqueta é só para a exceção e para achar quem vê numa lista.
 */
const COLOR = {
  /** Saiu, desistiu, ficou fora da chave. */
  cancelled: "red",
  /** Contestado. */
  contested: "magenta",
  /** A ação que falta a quem vê, no cartão da série. */
  action: "purple",
  /** Marcador sem gravidade: rascunho, "Você", "Sua série", reserva. */
  neutral: "gray",
} as const;

export type Status = keyof typeof COLOR;

type Props = {
  status: Status;
  /** O texto, já traduzido. */
  children: ReactNode;
  size?: "sm" | "md";
};

/**
 * Etiqueta de estado (`Tag` do Carbon) com a cor do significado. Só para o
 * que sai do caminho esperado (saiu, desistiu, contestado) e para achar quem
 * vê numa lista ("Você", "Esta sessão"). O caso esperado (inscrito, em
 * andamento, check-in feito, papel, vitória) é texto, sem marca; um processo
 * que pede cuidado sem ser erro (falta o check-in, empate a decidir) é o
 * `IconIndicator`. A situação de quem joga tem o seu mapa
 * (`SITUATION_MARK`, em `features/participants/lib/situation.ts`).
 *
 * @example <StatusTag status="cancelled">Desistiu</StatusTag>
 */
export function StatusTag({ status, children, size = "md" }: Props) {
  return (
    <Tag type={COLOR[status]} size={size}>
      {children}
    </Tag>
  );
}
