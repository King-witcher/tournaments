import { affects, type Interest } from "./interest";
import { RealtimeClient, type ServerMessage } from "./realtime-client";

/**
 * A conexão da aba e o que as páginas abertas mostram. Cada página declara o
 * que mostra (`watch`, pelo `LiveUpdates`): a comunidade ou o torneio viram
 * assinatura no socket, e o conjunto decide quais mudanças recarregam.
 * Guarda também quantas notificações faltam ver (`getUnread`), para o
 * sininho do cabeçalho.
 */
export class RealtimeHub {
  readonly client: RealtimeClient;
  private readonly interests = new Set<Interest>();
  private unread: number | undefined;
  private readonly unreadListeners = new Set<() => void>();

  constructor(url: string) {
    this.client = new RealtimeClient(url);
    this.client.onMessage((message) => {
      if (message.type === "unread") this.setUnread(message.count);
      else if (message.type === "notify")
        this.setUnread((this.unread ?? 0) + 1);
    });
  }

  /** A página passa a mostrar `interest`; devolve a função que desfaz. */
  watch(interest: Interest): () => void {
    this.interests.add(interest);
    const release = interest.tenant
      ? this.client.subscribe({
          tenant: interest.tenant,
          tournament: interest.tournament,
        })
      : undefined;
    return () => {
      this.interests.delete(interest);
      release?.();
    };
  }

  /** Se `message` muda algo que alguma página aberta mostra. */
  affectsPage(message: ServerMessage): boolean {
    for (const interest of this.interests) {
      if (affects(interest, message)) return true;
    }
    return false;
  }

  /** Se `message` muda algo que uma página acompanha ao vivo (`fast`). */
  affectsLive(message: ServerMessage): boolean {
    for (const interest of this.interests) {
      if (interest.fast && affects(interest, message)) return true;
    }
    return false;
  }

  /**
   * Quantas notificações faltam ver: a API manda ao conectar e quando a
   * pessoa as vê noutra aba (`unread`), e cada aviso (`notify`) soma uma.
   * `undefined` sem conexão. Estável, para o `useSyncExternalStore`.
   */
  getUnread = () => this.unread;

  /** Passa a ouvir as mudanças de `getUnread`; devolve a função que desfaz. */
  onUnread = (listener: () => void) => {
    this.unreadListeners.add(listener);
    return () => {
      this.unreadListeners.delete(listener);
    };
  };

  /**
   * A contagem que a API devolveu ao marcar como vistas, sem esperar o
   * socket; `undefined` ao fechar a conexão (a próxima pode ser de outra
   * conta).
   */
  setUnread(count: number | undefined) {
    if (count === this.unread) return;
    this.unread = count;
    for (const listener of this.unreadListeners) listener();
  }
}
