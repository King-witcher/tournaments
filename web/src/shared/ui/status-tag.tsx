import { Tag } from "@carbon/react";
import type { ReactNode } from "react";

/**
 * Cor fixa por significado (docs/design.md, "Estado"). Os códigos do torneio
 * (`draft`…`finished`) são os mesmos da API, para passar `tournament.status`
 * direto.
 */
const COLOR = {
  /** Rascunho. */
  draft: "gray",
  /** Inscrições abertas, inscrito. */
  registration: "blue",
  /** Check-in aberto ou pendente. */
  check_in: "purple",
  /** Em andamento, jogando. */
  in_progress: "green",
  /** Encerrado, eliminado. */
  finished: "cool-gray",
  /** Cancelado, desistiu, saiu, perdeu a vaga. */
  cancelled: "red",
  /** Aguardando o outro lado ou o prazo. */
  waiting: "teal",
  /** Contestado. */
  contested: "magenta",
  /** Confirmado, check-in feito, venceu. */
  confirmed: "green",
  /** Sem estado especial (papel, contagem). */
  neutral: "gray",
} as const;

export type Status = keyof typeof COLOR;

type Props = {
  status: Status;
  /** O nome do estado, já traduzido. */
  children: ReactNode;
  size?: "sm" | "md";
};

/** Etiqueta de estado (`Tag` do Carbon) com a cor do significado. */
export function StatusTag({ status, children, size = "md" }: Props) {
  return (
    <Tag type={COLOR[status]} size={size}>
      {children}
    </Tag>
  );
}
