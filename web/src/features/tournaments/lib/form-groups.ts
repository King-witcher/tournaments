/**
 * Onde um controle do formulário de torneio está escondido: o grupo
 * (acordeão na criação, aba na edição) e o idioma (aba de textos), lidos
 * dos `data-group` e `data-language` em volta dele. O formulário abre os
 * dois antes de levar o foco ao controle recusado.
 */
export function containersOf(control: Element) {
  return {
    group: control.closest<HTMLElement>("[data-group]")?.dataset.group,
    language: control.closest<HTMLElement>("[data-language]")?.dataset.language,
  };
}

/** O primeiro controle do formulário que a validação nativa recusa. */
export function firstInvalid(form: HTMLFormElement) {
  for (const element of Array.from(form.elements)) {
    const control = element as HTMLInputElement;
    if (control.willValidate && !control.validity.valid) return control;
  }
  return undefined;
}
