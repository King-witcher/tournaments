"use client";

import { InlineLoading } from "@carbon/react";

/**
 * O indicador girando do `InlineLoading`, sem texto, para ir dentro de outro
 * controle (o botão pendente). O `label` é lido por leitor de tela.
 */
export function Spinner({
  label,
  className,
}: {
  label: string;
  className?: string;
}) {
  return (
    <InlineLoading
      status="active"
      iconDescription={label}
      className={className}
    />
  );
}
