import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./legacy-field.module.scss";

/**
 * @deprecated Use `TextInput` (`labelText`, `helperText`, `invalid`,
 * `invalidText`), `PasswordInput` ou `NumberInput`. Este é o `<input>` nativo
 * com o visual do Carbon, para os formulários antigos dentro de `Field`.
 */
export function Input({ className, ...props }: ComponentProps<"input">) {
  return <input className={cn(styles.control, className)} {...props} />;
}
