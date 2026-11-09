import { type ReactNode, useId } from "react";

import { Stack } from "./carbon/stack";
import styles from "./section.module.scss";

type Props = {
  /** Título da seção (h2). */
  title: ReactNode;
  /** Uma frase embaixo do título, só com o que o título e o conteúdo não dizem. */
  description?: ReactNode;
  /** Ação da seção, à direita do título (no celular, embaixo). */
  action?: ReactNode;
  children: ReactNode;
  /** `id` do título, quando outro lugar precisa apontar para ele. */
  id?: string;
  /** Nível do título: 2 (padrão) ou 3, para a seção dentro de outra (`DangerZone`). */
  level?: 2 | 3;
  /** O título recebe o foco por script (`tabIndex={-1}`): o alvo de um "Ir para". */
  focusable?: boolean;
};

/**
 * Uma seção da página: título (h2 `heading-03`, ou h3 `heading-02`), descrição opcional, ação
 * ao lado e o conteúdo, com o espaçamento do Carbon. Uma coisa por seção.
 */
export function Section({
  title,
  description,
  action,
  children,
  id,
  level = 2,
  focusable = false,
}: Props) {
  const Heading = level === 3 ? "h3" : "h2";
  const generated = useId();
  const headingId = id ?? generated;
  return (
    <section aria-labelledby={headingId}>
      <Stack gap={5}>
        <div className={styles.header}>
          <Heading
            id={headingId}
            tabIndex={focusable ? -1 : undefined}
            className={level === 3 ? styles.subtitle : styles.title}
          >
            {title}
          </Heading>
          {action && <div className={styles.action}>{action}</div>}
          {description && <p className={styles.description}>{description}</p>}
        </div>
        {children}
      </Stack>
    </section>
  );
}
