/**
 * Põe `text` no lugar da seleção de um campo, numa linha só dele (quebra
 * antes e depois quando o vizinho não é uma), e seleciona `length`
 * caracteres a partir de `offset` dentro do texto (a descrição de uma
 * imagem, para trocar digitando). Pelo `insertText` do navegador, que
 * entra no desfazer (Ctrl+Z) e dispara o `input` como uma digitação; com o
 * campo escondido ou sem o comando, troca o valor e dispara o `input` à mão.
 * Nos dois casos, o `onChange` do React roda.
 */
export function insertAtCursor(
  field: HTMLTextAreaElement,
  text: string,
  offset = 0,
  length = 0,
) {
  const start = field.selectionStart;
  const end = field.selectionEnd;
  const before = field.value.slice(0, start);
  const after = field.value.slice(end);
  const lead = before && !before.endsWith("\n") ? "\n" : "";
  const trail = after && !after.startsWith("\n") ? "\n" : "";
  const insert = `${lead}${text}${trail}`;

  field.focus();
  field.setSelectionRange(start, end);
  const typed =
    document.activeElement === field &&
    document.execCommand("insertText", false, insert);
  if (!typed) {
    const setValue = Object.getOwnPropertyDescriptor(
      HTMLTextAreaElement.prototype,
      "value",
    )?.set;
    setValue?.call(field, before + insert + after);
    field.dispatchEvent(new Event("input", { bubbles: true }));
  }
  const from = start + lead.length + offset;
  field.setSelectionRange(from, from + length);
}
