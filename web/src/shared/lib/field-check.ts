/** O que pode estar errado com o texto de um campo. */
export type FieldProblem =
  | "required"
  | "minLength"
  | "maxLength"
  | "pattern"
  | "email"
  | "mismatch";

type Rules = {
  required?: boolean;
  minLength?: number;
  maxLength?: number;
  /** Como o atributo `pattern`: vale para o texto inteiro. */
  pattern?: string;
  type?: string;
  /** O texto tem de ser igual a este (a senha repetida). */
  sameAs?: string;
};

const EMAIL = /^[^@\s]+@[^@\s]+\.[^@\s]+$/;

/**
 * Confere o texto de um campo contra as regras que o próprio campo declara
 * (`required`, `minLength`, `maxLength`, `pattern`, `type="email"`). Campo
 * opcional e vazio está certo. Existe porque o navegador só confere
 * `minLength` e `pattern` depois de uma edição da pessoa, não de um valor
 * posto pelo código, como o endereço que acompanha o nome.
 */
export function checkField(
  value: string,
  rules: Rules,
): FieldProblem | undefined {
  if (value === "") return rules.required ? "required" : undefined;
  if (rules.minLength !== undefined && value.length < rules.minLength)
    return "minLength";
  if (rules.maxLength !== undefined && value.length > rules.maxLength)
    return "maxLength";
  if (rules.pattern && !new RegExp(`^(?:${rules.pattern})$`, "u").test(value))
    return "pattern";
  if (rules.type === "email" && !EMAIL.test(value)) return "email";
  if (rules.sameAs !== undefined && value !== rules.sameAs) return "mismatch";
  return undefined;
}
