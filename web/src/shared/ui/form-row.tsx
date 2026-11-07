import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./form-row.module.scss";

type Props = {
  /** Quantos campos cabem lado a lado, no máximo. */
  columns: 2 | 3 | 4;
  children: ReactNode;
};

/**
 * Campos de um formulário lado a lado, em colunas iguais alinhadas à calha da
 * grade do Carbon (o "two column form" dele): para campos curtos que se
 * decidem juntos (partidas e final, data e check-in) e para um campo curto
 * sozinho numa linha larga, que fica na primeira coluna em vez de esticar.
 * As colunas seguem a largura da linha, não a da tela: `columns={4}` dá
 * quatro na aba larga, duas num Modal ou numa coluna de 8 e uma no celular,
 * sem deixar um campo sozinho na última linha (4 cai para 2). Cada filho
 * direto é um item; um bloco que devolve vários campos num fragmento vira
 * vários itens. Um `Notice` ou um botão de linha inteira ficam fora da linha.
 */
export function FormRow({ columns, children }: Props) {
  return (
    <div className={styles.row}>
      <div className={cn(styles.grid, styles[`columns${columns}`])}>
        {children}
      </div>
    </div>
  );
}
