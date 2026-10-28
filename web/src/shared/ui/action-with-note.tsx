import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./action-with-note.module.scss";
import { ErrorText } from "./error-text";

type Props = {
  /** O botão (ou o menu ⋯). */
  children: ReactNode;
  /** Por que a ação não dá agora (o botão vem desabilitado). */
  note?: ReactNode;
  /** O erro da última tentativa, já traduzido (`useApiAction`). */
  error?: string;
  /** `end` na célula de ações de uma tabela: alinhado à direita. */
  align?: "start" | "end";
};

/**
 * Uma ação com, logo embaixo, o motivo de não dar ou o erro da última
 * tentativa: quem olha (ou ouve) o botão acha o porquê junto dele. O
 * invólucro existe sempre, para o botão não ser recriado (e perder o foco)
 * quando o erro aparece.
 */
export function ActionWithNote({
  children,
  note,
  error,
  align = "start",
}: Props) {
  return (
    <div className={cn(styles.action, align === "end" && styles.end)}>
      {children}
      {note && <p className={styles.note}>{note}</p>}
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </div>
  );
}
