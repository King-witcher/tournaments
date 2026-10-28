/** Junta nomes de classe (dos `.module.scss`), ignorando os falsos. */
export function cn(...classes: (string | false | null | undefined)[]) {
  return classes.filter(Boolean).join(" ");
}
