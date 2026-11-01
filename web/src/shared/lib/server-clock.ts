/**
 * A hora da API vista do navegador: quanto somar ao `Date.now()` daqui. Um
 * relógio atrasado mostraria um prazo com tempo que a API já não dá (o veto
 * só aceita até 2 s depois do prazo).
 *
 * Duas fontes, uma conta por aba para todos os prazos da página:
 * - a resposta de um pedido feito daqui (`noteServerTime` com `trip`): a hora
 *   da API cai no meio da ida e volta, então o erro é de meia viagem no
 *   máximo; vale a amostra da viagem mais curta;
 * - a hora em que o servidor montou a página (`noteServerTime` sem `trip`,
 *   pelo `serverNow` do `Countdown`): chega atrasada pelo tempo de
 *   renderizar, baixar e hidratar (no celular, 1 a 2 s), o que adiantaria o
 *   prazo na tela. Só vale para um relógio errado de verdade (diferença acima
 *   de `TRUSTED_MS`) e enquanto não há amostra de pedido; entre elas, a
 *   maior (todas ficam atrás da hora real, nunca à frente).
 */
const TRUSTED_MS = 3000;

let trip: { offset: number; length: number } | undefined;
let rendered: number | undefined;

/**
 * Registra a hora da API (ISO). `sent` e `received`: quando o pedido saiu e
 * quando a resposta chegou (`Date.now()`), numa resposta a um pedido feito
 * daqui; sem eles, é a hora da renderização da página.
 */
export function noteServerTime(
  iso: string,
  timing?: { sent: number; received: number },
) {
  const server = Date.parse(iso);
  if (Number.isNaN(server)) return;
  if (timing) {
    const length = timing.received - timing.sent;
    if (trip === undefined || length <= trip.length) {
      trip = { offset: server - (timing.sent + timing.received) / 2, length };
    }
    return;
  }
  const sample = server - Date.now();
  if (Math.abs(sample) <= TRUSTED_MS) return;
  if (rendered === undefined || sample > rendered) rendered = sample;
}

/** A hora da API agora, em ms; sem amostra que valha, a daqui. */
export function serverTime() {
  return Date.now() + (trip?.offset ?? rendered ?? 0);
}

/** Esquece as amostras (histórias e testes que simulam outro relógio). */
export function resetServerClock() {
  trip = undefined;
  rendered = undefined;
}
