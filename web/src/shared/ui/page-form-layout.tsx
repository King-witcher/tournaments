import type { ReactNode } from "react";

import { Stack } from "./carbon/stack";
import styles from "./page-form-layout.module.scss";

type Props = {
  /** Os campos, um embaixo do outro. */
  children: ReactNode;
  /** O erro geral, o "Salvo." e os botões; sem nada, nem o espaço deles. */
  footer?: ReactNode;
};

/**
 * O espaçamento de um formulário de página, o que o Carbon recomenda: 32px
 * entre os campos e 48px entre o último e os botões. Modal e painel lateral
 * ficam no `Stack gap={6}`, os 24px que o Carbon aceita em formulário contido.
 *
 * ```tsx
 * <Form action={submit}>
 *   <PageFormLayout footer={<FormActions pending={pending} />}>
 *     <TextInput labelText="Nome" />
 *   </PageFormLayout>
 * </Form>
 * ```
 */
export function PageFormLayout({ children, footer }: Props) {
  return (
    <Stack gap={7} className={styles.layout}>
      {children}
      {footer && (
        <Stack gap={5} className={styles.footer}>
          {footer}
        </Stack>
      )}
    </Stack>
  );
}
