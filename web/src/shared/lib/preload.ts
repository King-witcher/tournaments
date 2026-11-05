/**
 * Começa uma busca do servidor sem esperar por ela, antes de um `await` que
 * atrasaria quem a usa mais abaixo: `preload(getMyTenants())`. Só serve para
 * função com `cache()` do React chamada com **os mesmos argumentos** do uso
 * (`===`), que recebe a mesma promessa; com outros, a busca sai duas vezes. E
 * só para busca que a página faria de qualquer jeito: com o resultado
 * descartado (sem sessão, erro antes), vira pedido a mais.
 *
 * O `catch` vazio não esconde o erro de quem usa, que o recebe no próprio
 * `await`: só evita o `unhandledRejection` quando ninguém chega a esperar a
 * promessa (a página parou antes, ou o prerender abortou o `cookies()`).
 */
export function preload(promise: Promise<unknown>): void {
  promise.catch(() => {});
}
