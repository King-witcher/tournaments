import { affects, type Interest } from "./interest";
import { RealtimeClient, type ServerMessage } from "./realtime-client";

/**
 * A conexão da aba e o que as páginas abertas mostram. Cada página declara o
 * que mostra (`watch`, pelo `LiveUpdates`): a comunidade ou o torneio viram
 * assinatura no socket, e o conjunto decide quais mudanças recarregam.
 */
export class RealtimeHub {
  readonly client: RealtimeClient;
  private readonly interests = new Set<Interest>();

  constructor(url: string) {
    this.client = new RealtimeClient(url);
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
}
