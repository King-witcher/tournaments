/**
 * O grupo que esconde um controle do formulário de torneio (acordeão na
 * criação, aba na edição), lido do `data-group` em volta dele. O formulário
 * o abre antes de levar o foco ao controle recusado.
 */
export function groupOf(control: Element) {
  return control.closest<HTMLElement>("[data-group]")?.dataset.group;
}

/** O primeiro controle do formulário que a validação nativa recusa. */
export function firstInvalid(form: HTMLFormElement) {
  for (const element of Array.from(form.elements)) {
    const control = element as HTMLInputElement;
    if (control.willValidate && !control.validity.valid) return control;
  }
  return undefined;
}
