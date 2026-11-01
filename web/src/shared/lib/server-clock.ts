/**
 * A hora da API vista do navegador: quanto somar ao `Date.now()` daqui. Um
 * relógio atrasado mostraria um prazo com tempo que a API já não dá (o veto
 * só aceita até 2 s depois do prazo).
 *
 * Cada resposta que traz a hora da API (`now` do `MatchDetail`) é uma
 * amostra, registrada quando chega (`noteServerTime`). A resposta leva um
 * tempo para chegar e a página para aparecer, então toda amostra fica atrás
 * da hora real por essa demora, nunca à frente: a maior é a mais certa, e
 * uma velha (a página que o "voltar" do navegador traz de novo) não piora a
 * conta. Uma por aba, para todos os prazos da página.
 */
let offset: number | undefined;

/** Registra a hora da API (ISO) de uma resposta que acabou de chegar. */
export function noteServerTime(iso: string) {
  const sample = Date.parse(iso) - Date.now();
  if (Number.isNaN(sample)) return;
  if (offset === undefined || sample > offset) offset = sample;
}

/** A hora da API agora, em ms; sem amostra, a daqui. */
export function serverTime() {
  return Date.now() + (offset ?? 0);
}

/** Esquece as amostras (histórias e testes que simulam outro relógio). */
export function resetServerClock() {
  offset = undefined;
}
