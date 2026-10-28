import { Stack } from "@carbon/react";
import { type ReactNode, useId } from "react";

import styles from "./section.module.scss";

type Props = {
  /** Título da seção (h2). */
  title: ReactNode;
  /** Uma frase sobre a seção, embaixo do título. */
  description?: ReactNode;
  /** Ação da seção, à direita do título (no celular, embaixo). */
  action?: ReactNode;
  children: ReactNode;
  /** `id` do título, quando outro lugar precisa apontar para ele. */
  id?: string;
};

/**
 * Uma seção da página: título (h2, `heading-03`), descrição opcional, ação
 * ao lado e o conteúdo, com o espaçamento do Carbon. Uma coisa por seção.
 */
export function Section({ title, description, action, children, id }: Props) {
  const generated = useId();
  const headingId = id ?? generated;
  return (
    <section aria-labelledby={headingId}>
      <Stack gap={5}>
        <div className={styles.header}>
          <h2 id={headingId} className={styles.title}>
            {title}
          </h2>
          {action}
          {description && <p className={styles.description}>{description}</p>}
        </div>
        {children}
      </Stack>
    </section>
  );
}
