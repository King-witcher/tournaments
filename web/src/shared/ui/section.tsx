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
  /** Nível do título: 2 (padrão) ou 3, para a seção dentro de outra (`DangerZone`). */
  level?: 2 | 3;
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
            className={level === 3 ? styles.subtitle : styles.title}
          >
            {title}
          </Heading>
          {action}
          {description && <p className={styles.description}>{description}</p>}
        </div>
        {children}
      </Stack>
    </section>
  );
}
