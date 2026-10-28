/** Campo de um formulário de conta (entrar, cadastro, senha, nome, email). */
export type AuthField =
  | "username"
  | "email"
  | "password"
  | "confirm"
  | "current"
  | "name";

/** Os erros da API que são de um campo só; o resto é do formulário todo. */
const FIELD_OF_ERROR: Partial<Record<string, AuthField>> = {
  "auth.username_invalid": "username",
  "auth.username_taken": "username",
  "auth.password_invalid": "password",
  "auth.wrong_password": "current",
  "auth.display_name_invalid": "name",
  "auth.email_taken": "email",
  "auth.email_unchanged": "email",
  "email.invalid": "email",
  "email.disposable": "email",
};

/** O campo a que o erro `code` da API se refere, se for de um campo. */
export function errorField(code: string) {
  return FIELD_OF_ERROR[code];
}
