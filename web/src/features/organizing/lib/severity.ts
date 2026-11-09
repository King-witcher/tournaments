import type { components } from "@/shared/api/schema";

/** O que pode esperar quem vê. `resolve`: o mapa contestado. */
type Pending =
  | components["schemas"]["ViewerTask"]
  | "pending_ties"
  | "start_due"
  | "stage_due"
  | "check_in";

export type Severity = "warning" | "info";

/**
 * `warning` é a exceção que trava alguém e não acontece no fluxo normal
 * (contestação, empate a decidir); o resto é o próximo passo esperado, `info`.
 * Veto e ready ficam em `info`: são a rotina de quem joga, e amarelo em tudo
 * deixaria de chamar atenção. A API não tem severidade: é só apresentação.
 */
const WARNING: Pending[] = ["resolve", "accept_contest", "pending_ties"];

export function severity(pending: Pending): Severity {
  return WARNING.includes(pending) ? "warning" : "info";
}

/** Warnings primeiro; dentro de cada grupo, a ordem de antes (`toSorted` é estável). */
export function warningsFirst<T extends { kind: Severity }>(items: T[]) {
  return items.toSorted(
    (a, b) => Number(b.kind === "warning") - Number(a.kind === "warning"),
  );
}
