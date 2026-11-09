import type { ReactNode } from "react";

import { Layer } from "./carbon/layer";
import styles from "./panel.module.scss";

type Props = {
  children: ReactNode;
};

/**
 * Um container branco sobre a página cinza (`g10`), como as seções dos
 * produtos da IBM e da AWS: texto, formulário e lista de rótulo e valor ficam
 * dentro dele, e o que o Carbon desenha por cima (campo, tile de opção, tag,
 * `pre`) passa para a camada seguinte, que contrasta com o branco, porque o
 * `Layer` já vem no `Panel`. Um por assunto, com o título (`Section`) dentro,
 * e o esqueleto do carregamento também. Tabela, chave, lista de tiles e as
 * grades do veto têm o fundo próprio e ficam soltas na página; título, trilha
 * e ações são da página. Nunca um `Panel` dentro de outro. No escuro é a
 * camada `g100` mais clara que a página. Entre dois, `Stack gap={7}`, o mesmo
 * espaço das colunas.
 *
 * ```tsx
 * <Panel>
 *   <Section title="Como funciona">…</Section>
 * </Panel>
 * ```
 */
export function Panel({ children }: Props) {
  return <Layer className={styles.panel}>{children}</Layer>;
}
