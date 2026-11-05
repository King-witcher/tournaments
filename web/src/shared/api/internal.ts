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
