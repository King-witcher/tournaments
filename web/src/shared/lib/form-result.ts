/**
 * O resultado de um envio: o erro já traduzido e, quando ele é de um campo,
 * qual (`F` são os nomes dos campos do formulário). O erro de um campo vai no
 * `invalidText` dele (`invalidProps`); sem campo, no `FormError`
 * (`formError`). `saved` diz que salvou, nos formulários que ficam na tela.
 */
export type FormResult<F extends string = string> = {
  error?: string;
  field?: F;
  saved?: boolean;
};

/** As props de erro do campo `field` (`invalid`, `invalidText`) para o `result`. */
export function invalidProps<F extends string>(
  result: FormResult<F>,
  field: F,
) {
  return result.field === field
    ? { invalid: true, invalidText: result.error }
    : {};
}

/** O erro do formulário todo (o que vai no `FormError`), se houver. */
export function formError(result: FormResult) {
  return result.field ? undefined : result.error;
}

/** O id do campo `field` num formulário com o `prefix` do `useId`. */
export function fieldId(prefix: string, field: string) {
  return `${prefix}${field}`;
}
