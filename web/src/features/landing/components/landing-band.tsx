import { Grid } from "@carbon/react";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./landing-band.module.scss";

type Props = {
  /** As `Column`s da faixa, na grade de 16 colunas. */
  children: ReactNode;
  /** `id` do título (h2) da faixa. */
  labelledBy: string;
  /** `layer`: fundo de camada, para a faixa se separar das vizinhas. */
  tone?: "background" | "layer";
};

/**
 * Uma faixa da apresentação: seção de parede a parede (o fundo vai até as
 * bordas da janela) com a grade do Carbon dentro, que mantém as margens.
 */
export function LandingBand({
  children,
  labelledBy,
  tone = "background",
}: Props) {
  return (
    <section
      aria-labelledby={labelledBy}
      className={cn(styles.band, tone === "layer" && styles.layer)}
    >
      <Grid>{children}</Grid>
    </section>
  );
}
