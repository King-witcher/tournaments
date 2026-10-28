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
