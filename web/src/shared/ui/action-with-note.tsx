"use client";

import { Tooltip } from "@carbon/react";
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
  /**
   * Numa linha de botões (o cabeçalho): o `note` vira a dica do botão, ao
   * passar o mouse, focar ou tocar. Escrito embaixo, ele desalinharia os
   * vizinhos. O erro continua embaixo, porque precisa ser visto.
   */
  compact?: boolean;
  /** Com `compact`: o botão está `disabled`, então a dica abre pelo invólucro. */
  blocked?: boolean;
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
  compact,
  blocked,
}: Props) {
  return (
    <div className={cn(styles.action, align === "end" && styles.end)}>
      {compact ? (
        <ButtonTip tip={note} blocked={blocked}>
          {children}
        </ButtonTip>
      ) : (
        children
      )}
      {!compact && note && <p className={styles.note}>{note}</p>}
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </div>
  );
}

type TipProps = {
  children: ReactNode;
  tip?: ReactNode;
  blocked?: boolean;
};

/**
 * `disabled` não recebe mouse nem foco: com `blocked`, o invólucro é que
 * recebe, entra na ordem do Tab e abre a dica ao tocar. Existe com ou sem
 * `tip`, para o botão não ser recriado quando o motivo aparece ou some.
 */
function ButtonTip({ children, tip, blocked }: TipProps) {
  const focusable = blocked && Boolean(tip);
  return (
    <Tooltip
      className={tip ? undefined : styles.empty}
      description={tip || ""}
      align="bottom"
      autoAlign
    >
      <span
        className={cn(styles.trigger, blocked && styles.blocked)}
        role={focusable ? "group" : undefined}
        tabIndex={focusable ? 0 : undefined}
      >
        {children}
      </span>
    </Tooltip>
  );
}
