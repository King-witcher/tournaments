import { useTranslations } from "next-intl";
import {
  type ChangeEvent,
  type FocusEvent,
  type ReactNode,
  useEffect,
  useRef,
  useState,
} from "react";

import { checkField } from "@/shared/lib/field-check";

type Element = HTMLInputElement | HTMLTextAreaElement;

type Options<E extends Element> = {
  required?: boolean;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  type?: string;
  /** O texto tem de ser igual a este (senha repetida). */
  sameAs?: string;
  /** Erro do `pattern`; sem ela, "Formato inválido". */
  patternMessage?: ReactNode;
  /** Tira do texto o que o campo não aceita, ao digitar ou colar. */
  filter?: (value: string) => string;
  /** Erro vindo de fora (o que a API devolveu): vale mais que o daqui. */
  invalid?: boolean;
  invalidText?: ReactNode;
  onChange?: (event: ChangeEvent<E>) => void;
  onBlur?: (event: FocusEvent<E>) => void;
};

/**
 * A validação de um campo de texto: aplica o `filter`, confere as regras do
 * campo (`checkField`) e as grava no próprio elemento (`setCustomValidity`),
 * para o `form.checkValidity()` (ver `useFormValidity`) saber de todas. O
 * erro só aparece depois que a pessoa sai do campo pela primeira vez, para
 * "mínimo de 3" não ficar vermelho na primeira letra; dali em diante, a cada
 * tecla. Devolve `ref`, `onChange`, `onBlur`, `invalid` e `invalidText` para
 * pôr no input.
 */
export function useFieldCheck<E extends Element>(options: Options<E>) {
  const t = useTranslations("Common.validation");
  const ref = useRef<E>(null);
  const [problem, setProblem] = useState<ReturnType<typeof checkField>>();
  const [touched, setTouched] = useState(false);
  const { required, minLength, maxLength, pattern, type, sameAs, filter } =
    options;

  // A cada render: o valor pode ter vindo do código, sem evento nenhum.
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const found = checkField(element.value, {
      required,
      minLength,
      maxLength,
      pattern,
      type,
      sameAs,
    });
    element.setCustomValidity(found ?? "");
    setProblem(found);
  });

  const shown = touched ? problem : undefined;
  const messages = {
    required: t("required"),
    minLength: t("minLength", { min: minLength ?? 0 }),
    maxLength: t("maxLength", { max: maxLength ?? 0 }),
    pattern: options.patternMessage ?? t("pattern"),
    email: t("email"),
    mismatch: t("mismatch"),
  };

  return {
    ref,
    onChange(event: ChangeEvent<E>) {
      const element = event.target;
      const clean = filter?.(element.value);
      if (clean !== undefined && clean !== element.value) {
        // Sem isto o cursor pularia para o fim a cada caractere recusado.
        const caret = element.selectionStart;
        const removed = element.value.length - clean.length;
        element.value = clean;
        if (caret !== null && element.type === "text") {
          element.setSelectionRange(
            Math.max(0, caret - removed),
            Math.max(0, caret - removed),
          );
        }
      }
      options.onChange?.(event);
    },
    onBlur(event: FocusEvent<E>) {
      setTouched(true);
      options.onBlur?.(event);
    },
    invalid: Boolean(options.invalid || shown),
    invalidText: options.invalid
      ? options.invalidText
      : shown && messages[shown],
  };
}
