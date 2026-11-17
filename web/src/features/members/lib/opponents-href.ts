/** A modalidade da aba Confrontos; sem ela, todas. */
export type Modality = "duel" | "tdm";

/** O endereço da aba Confrontos com a modalidade (todas fica de fora). */
export function opponentsHref(base: string, modality?: Modality) {
  return modality
    ? `${base}/opponents?modality=${modality}`
    : `${base}/opponents`;
}

/** A modalidade lida do endereço; o que não vale é todas. */
export function parseModality(
  value: string | string[] | undefined,
): Modality | undefined {
  return value === "duel" || value === "tdm" ? value : undefined;
}
