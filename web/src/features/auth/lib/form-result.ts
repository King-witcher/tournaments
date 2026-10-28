/** Campo de um formulário de conta (entrar, cadastro, senha, nome). */
export type AuthField =
  | "username"
  | "password"
  | "confirm"
  | "current"
  | "name";

/**
 * O resultado de um envio: o erro já traduzido e, quando ele é de um campo,
 * qual (o erro vai no `invalidText` dele; sem campo, no `FormError`).
 * `saved` diz que salvou, nos formulários que ficam na tela.
 */
export type FormResult = {
  error?: string;
  field?: AuthField;
  saved?: boolean;
};

/** Os erros da API que são de um campo só; o resto é do formulário todo. */
const FIELD_OF_ERROR: Partial<Record<string, AuthField>> = {
  "auth.username_invalid": "username",
  "auth.username_taken": "username",
  "auth.password_invalid": "password",
  "auth.wrong_password": "current",
  "auth.display_name_invalid": "name",
};

/** O campo a que o erro `code` da API se refere, se for de um campo. */
export function errorField(code: string) {
  return FIELD_OF_ERROR[code];
}

/** As props de erro do campo `field` (`invalid`, `invalidText`) para o `result`. */
export function invalidProps(result: FormResult, field: AuthField) {
  return result.field === field
    ? { invalid: true, invalidText: result.error }
    : {};
}

/** O erro do formulário todo (o que vai no `FormError`), se houver. */
export function formError(result: FormResult) {
  return result.field ? undefined : result.error;
}
