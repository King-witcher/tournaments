/**
 * Leva o foco a um controle que acabou de ser mostrado (o acordeão ou a aba
 * dele abriu agora e o navegador ainda não o desenhou): tenta a cada quadro,
 * por pouco tempo, até ele ficar visível. `then` roda depois do foco (ex.:
 * `reportValidity`, que só mostra a mensagem num controle visível).
 */
export function focusWhenShown(element: HTMLElement, then?: () => void) {
  const until = performance.now() + 1000;
  const tick = () => {
    if (element.checkVisibility()) {
      element.focus();
      then?.();
      return;
    }
    if (performance.now() < until) requestAnimationFrame(tick);
  };
  tick();
}

/**
 * Depois de uma ação cujo controle some quando a página recarrega os dados
 * (o "Confirmar" de um mapa que ficou confirmado), leva o foco para o
 * elemento `id` se ele caiu no `body`. Tenta a cada quadro por alguns
 * segundos, até o refresh trocar o conteúdo; se a pessoa já foi para outro
 * controle, não mexe.
 */
export function focusIfLost(id: string) {
  const until = performance.now() + 3000;
  const tick = () => {
    const active = document.activeElement;
    const target = document.getElementById(id);
    if ((active == null || active === document.body) && target) {
      target.focus();
      return;
    }
    if (performance.now() < until) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}
