import {
  StructuredListBody,
  StructuredListCell,
  StructuredListHead,
  StructuredListRow,
  StructuredListWrapper,
} from "@carbon/react";
import type { ReactNode } from "react";

type Row = {
  key: string;
  cells: ReactNode[];
};

type Props = {
  /** Nome da lista para leitor de tela. */
  label: string;
  /** Cabeçalho das colunas; sem ele, a lista é de pares rótulo/valor. */
  headers?: ReactNode[];
  rows: Row[];
  /** Linhas mais baixas (resumo com muitos itens). */
  condensed?: boolean;
};

/**
 * Lista em colunas (`StructuredList` do Carbon), só leitura: o resumo do
 * formato (rótulo e valor por linha) e os mapas de uma partida. Para lista
 * com busca, paginação ou ação por linha, `DataTable`.
 */
export function StructuredList({ label, headers, rows, condensed }: Props) {
  return (
    <StructuredListWrapper aria-label={label} isCondensed={condensed}>
      {headers && (
        <StructuredListHead>
          <StructuredListRow head>
            {headers.map((header, index) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: colunas fixas, na ordem.
              <StructuredListCell key={index} head>
                {header}
              </StructuredListCell>
            ))}
          </StructuredListRow>
        </StructuredListHead>
      )}
      <StructuredListBody>
        {rows.map((row) => (
          <StructuredListRow key={row.key}>
            {row.cells.map((cell, index) => (
              <StructuredListCell
                // biome-ignore lint/suspicious/noArrayIndexKey: colunas fixas, na ordem.
                key={index}
                noWrap={!headers && index === 0}
              >
                {cell}
              </StructuredListCell>
            ))}
          </StructuredListRow>
        ))}
      </StructuredListBody>
    </StructuredListWrapper>
  );
}
