import type { components } from "@/shared/api/schema";

export type ServerMessage = components["schemas"]["ServerMessage"];
type ClientMessage = components["schemas"]["ClientMessage"];

/** O que uma aba assina: a comunidade, ou um torneio dela. */
export type Topic = { tenant: string; tournament?: string };

/** Espera antes de cada tentativa de reconectar; a última se repete. */
const BACKOFF_MS = [1000, 2000, 5000, 10000, 30000];
/**
 * A espera com a aba escondida, que cresce até 5 min: cada tentativa lê a
 * sessão no Redis, e uma aba esquecida com a sessão vencida tentaria para
 * sempre (a cada 5 min, ~290 comandos por dia; a cada 30 s, ~2.900).
 */
const HIDDEN_BACKOFF_MS = [1000, 5000, 30000, 120000, 300000];
/** Fechamento da API quando a sessão acaba (sair, encerrar, trocar a senha). */
const SESSION_ENDED = 4001;
/**
 * Quanto uma assinatura sem página espera antes de sair: ao navegar, a página
 * nova costuma pedir a mesma (da chave para a partida), e assim ela não sai e
 * volta.
 */
const RELEASE_DELAY_MS = 2000;

type Entry = {
  topic: Topic;
  count: number;
  release?: ReturnType<typeof setTimeout>;
};

const topicKey = ({ tenant, tournament }: Topic) =>
  tournament ? `${tenant}/${tournament}` : tenant;

/**
 * A conexão de uma aba com `GET /ws`, sem React. Conta as assinaturas por
 * tópico (duas páginas iguais, uma assinatura), assina de novo ao reconectar
 * e reconecta com espera crescente, menos depois do fechamento 4001 (sessão
 * encerrada). Com a aba escondida também reconecta, para os avisos chegarem a
 * quem deixou o site numa aba de lado, mas esperando mais
 * (`HIDDEN_BACKOFF_MS`); ao voltar a ser vista, a tentativa longa marcada
 * enquanto ela estava escondida sai na hora.
 */
export class RealtimeClient {
  private socket?: WebSocket;
  private readonly topics = new Map<string, Entry>();
  private readonly listeners = new Set<(message: ServerMessage) => void>();
  private readonly reconnectListeners = new Set<() => void>();
  private active = false;
  /** Tentativas seguidas sem abrir. */
  private attempt = 0;
  private opened = false;
  private retry?: ReturnType<typeof setTimeout>;
  /** A tentativa marcada foi marcada com a aba escondida (espera longa). */
  private retryWhileHidden = false;

  constructor(private readonly url: string) {}

  /** Abre a conexão (e mantém aberta até `close`). */
  connect() {
    if (this.active) return;
    this.active = true;
    document.addEventListener("visibilitychange", this.onVisibility);
    this.open();
  }

  /** Fecha e não reconecta. */
  close() {
    this.active = false;
    this.opened = false;
    this.attempt = 0;
    clearTimeout(this.retry);
    this.retry = undefined;
    document.removeEventListener("visibilitychange", this.onVisibility);
    const socket = this.socket;
    this.socket = undefined;
    socket?.close(1000);
  }

  /** Passa a receber as mudanças de `topic`; devolve a função que desfaz. */
  subscribe(topic: Topic): () => void {
    const key = topicKey(topic);
    let entry = this.topics.get(key);
    if (entry) {
      clearTimeout(entry.release);
      entry.release = undefined;
    } else {
      entry = { topic, count: 0 };
      this.topics.set(key, entry);
      this.send({ type: "subscribe", ...wire(topic) });
    }
    entry.count++;
    const current = entry;
    let released = false;
    return () => {
      if (released) return;
      released = true;
      current.count--;
      if (current.count > 0) return;
      current.release = setTimeout(() => {
        if (current.count > 0 || this.topics.get(key) !== current) return;
        this.topics.delete(key);
        this.send({ type: "unsubscribe", ...wire(topic) });
      }, RELEASE_DELAY_MS);
    };
  }

  onMessage(listener: (message: ServerMessage) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /** Chamado ao reabrir depois de uma queda: o que mudou no meio se perdeu. */
  onReconnect(listener: () => void): () => void {
    this.reconnectListeners.add(listener);
    return () => this.reconnectListeners.delete(listener);
  }

  private open() {
    if (!this.active) return;
    let socket: WebSocket;
    try {
      socket = new WebSocket(this.url);
    } catch {
      this.scheduleRetry();
      return;
    }
    this.socket = socket;
    socket.onopen = () => {
      const reconnected = this.opened || this.attempt > 0;
      this.opened = true;
      this.attempt = 0;
      for (const { topic } of this.topics.values()) {
        this.send({ type: "subscribe", ...wire(topic) });
      }
      if (!reconnected) return;
      for (const listener of this.reconnectListeners) listener();
    };
    socket.onmessage = (event) => {
      let message: ServerMessage;
      try {
        message = JSON.parse(String(event.data));
      } catch {
        return;
      }
      for (const listener of this.listeners) listener(message);
    };
    socket.onclose = (event) => {
      if (this.socket !== socket) return;
      this.socket = undefined;
      if (event.code === SESSION_ENDED) {
        this.active = false;
        return;
      }
      this.scheduleRetry();
    };
  }

  private scheduleRetry() {
    if (!this.active) return;
    this.retryWhileHidden = document.visibilityState === "hidden";
    const steps = this.retryWhileHidden ? HIDDEN_BACKOFF_MS : BACKOFF_MS;
    const wait = steps[Math.min(this.attempt, steps.length - 1)];
    this.attempt++;
    this.retry = setTimeout(() => {
      this.retry = undefined;
      this.open();
    }, wait);
  }

  private onVisibility = () => {
    if (document.visibilityState !== "visible") return;
    if (this.retry === undefined || !this.retryWhileHidden) return;
    clearTimeout(this.retry);
    this.retry = undefined;
    this.open();
  };

  private send(message: ClientMessage) {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(message));
    }
  }
}

function wire({ tenant, tournament }: Topic) {
  return tournament ? { tenant, tournament } : { tenant };
}
