import type { ReactNode } from "react";

type Props = {
  label: string;
  hint?: string;
  children: ReactNode;
};

/** Rótulo, controle e dica opcional num `<label>`: clicar no texto foca o controle. */
export function Field({ label, hint, children }: Props) {
  return (
    // biome-ignore lint/a11y/noLabelWithoutControl: o controle chega por `children`, dentro do label.
    <label className="flex flex-col gap-1">
      <span className="text-sm font-medium">{label}</span>
      {children}
      {hint && <span className="text-sm text-muted">{hint}</span>}
    </label>
  );
}
