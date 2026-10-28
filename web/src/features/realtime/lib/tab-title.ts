/**
 * O título da aba enquanto chegam avisos com ela escondida ("(2) Sua partida
 * está pronta"): é o que aparece na barra de abas de quem deixou o site de
 * lado. `restoreTabTitle` devolve o da página, menos se o Next o trocou no
 * meio tempo (aí o novo fica). Só no navegador.
 */

let pageTitle: string | undefined;
let shown: string | undefined;

export function showTabTitle(title: string) {
  if (document.title !== shown) pageTitle = document.title;
  shown = title;
  document.title = title;
}

export function restoreTabTitle() {
  if (shown !== undefined && document.title === shown && pageTitle) {
    document.title = pageTitle;
  }
  shown = undefined;
  pageTitle = undefined;
}
