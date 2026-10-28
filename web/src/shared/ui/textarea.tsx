import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./legacy-field.module.scss";

/**
 * @deprecated Use `TextArea` (de `./text-area`), com `labelText` e
 * `helperText`. Este é o `<textarea>` nativo com o visual do Carbon.
 */
export function Textarea({ className, ...props }: ComponentProps<"textarea">) {
  return (
    <textarea
      className={cn(styles.control, styles.multiline, className)}
      {...props}
    />
  );
}
