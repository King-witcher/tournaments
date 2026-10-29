"use client";

import { Information } from "@carbon/icons-react";
import { Toggletip, ToggletipButton, ToggletipContent } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./info-tip.module.scss";

type Props = {
  /** Nome do botão para leitor de tela, como a pergunta que a dica responde: "O que é check-in?". */
  label: string;
  /** O termo explicado, mostrado antes do (i) e preso a ele (não quebram linha entre si). Sem ele, só o (i). */
  term?: ReactNode;
  /** A explicação: uma a três frases curtas, sem link nem ação. */
  children: ReactNode;
};

/**
 * Um (i) ao lado de um termo que quem joga pode não conhecer ("Check-in",
 * "Eliminação dupla", "Prazo para contestar"), que abre uma explicação curta
 * ao clicar ou tocar (`Toggletip` do Carbon): funciona no celular, que não
 * tem hover, e no teclado (Enter abre; Esc fecha e devolve o foco ao (i)).
 * A caixa escolhe o lado em que cabe (`autoAlign`), para não sair da tela no
 * celular; lá, a área de toque do (i) tem 48px.
 *
 * Use em rótulo cujo sentido não é óbvio para quem não conhece o termo. O que
 * todo mundo precisa ler vai no texto da página, não numa dica escondida.
 *
 * ```tsx
 * <InfoTip term={t("labels.checkIn")} label={t("tips.checkIn")}>
 *   {t("tips.checkInText")}
 * </InfoTip>
 * ```
 */
export function InfoTip({ label, term, children }: Props) {
  const tip = (
    <Toggletip align="bottom-start" autoAlign>
      <ToggletipButton label={label} className={styles.button}>
        <Information />
      </ToggletipButton>
      <ToggletipContent className={styles.content}>
        <p>{children}</p>
      </ToggletipContent>
    </Toggletip>
  );
  if (term === undefined) return tip;
  return (
    <span className={styles.term}>
      {term}
      {tip}
    </span>
  );
}
