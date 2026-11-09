/**
 * Identifica o servidor do Next para a API (`scanners/` nela): os pedidos dele
 * saem pelos IPs da Vercel, que outros clientes dela também usam, e não podem ser
 * barrados por um ban. Todo pedido do servidor à API leva isto; o segredo não
 * tem `NEXT_PUBLIC_` e não chega ao navegador.
 */
export function internalHeaders(): Record<string, string> {
  const secret = process.env.INTERNAL_API_SECRET;
  return secret ? { "x-internal-secret": secret } : {};
}

/**
 * O IP de quem abriu a página, para a API contar nele o limite por IP em vez de
 * no IP da Vercel (`invites/limit.rs` na API, que só aceita com o segredo acima).
 * O `x-real-ip`, a Vercel grava por cima do que o cliente manda. Sem ele
 * (`next dev`), nada.
 */
export function viewerIpHeaders(request: Pick<Headers, "get">) {
  const ip = request.get("x-real-ip");
  return ip ? { "x-viewer-ip": ip } : undefined;
}
