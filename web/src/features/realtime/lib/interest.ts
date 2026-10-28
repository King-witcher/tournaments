import type { ServerMessage } from "./realtime-client";

/**
 * O que uma página mostra, para saber quais mudanças a recarregam: a
 * comunidade (`tenant`), um torneio dela (`tournament`), uma partida dele
 * (`match`) e "Seus torneios" (`me`).
 */
export type Interest = {
  tenant?: string;
  tournament?: string;
  match?: string;
  me?: boolean;
};

/**
 * Se `message` pede para recarregar uma página que mostra `interest`. A
 * comunidade recarrega pela mudança dela, e as páginas do torneio também
 * (apelidos e papéis aparecem na chave e nas ações). O torneio recarrega por
 * qualquer mudança dele; a partida, só pela própria (ou pela do torneio
 * inteiro, `match` nulo). Perder o acesso à comunidade recarrega as páginas
 * dela (caem no "sem acesso").
 */
export function affects(interest: Interest, message: ServerMessage): boolean {
  if (message.type === "access_lost") return interest.tenant === message.tenant;
  if (message.type !== "changed") return false;
  switch (message.scope) {
    case "me":
      return interest.me === true;
    case "tenant":
      return interest.tenant === message.tenant;
    case "tournament":
      return (
        interest.tenant === message.tenant &&
        interest.tournament === message.tournament &&
        (interest.match === undefined ||
          message.match == null ||
          message.match === interest.match)
      );
  }
}
