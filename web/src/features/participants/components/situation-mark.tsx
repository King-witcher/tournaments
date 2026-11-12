import type { components } from "@/shared/api/schema";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { StatusTag } from "@/shared/ui/status-tag";
import { SITUATION_MARK } from "../lib/situation";

type Props = {
  situation: components["schemas"]["Situation"];
  /** O nome da situação, já traduzido. */
  label: string;
};

/**
 * A situação de alguém num torneio: a exceção com marca (etiqueta ou o
 * indicador de cuidado, pelo `SITUATION_MARK`) e o caminho esperado em texto
 * simples, que não chama atenção.
 */
export function SituationMark({ situation, label }: Props) {
  const mark = SITUATION_MARK[situation];
  if (mark === "caution") {
    return <IconIndicator kind="caution-minor" label={label} />;
  }
  if (mark) return <StatusTag status={mark}>{label}</StatusTag>;
  return <span>{label}</span>;
}
