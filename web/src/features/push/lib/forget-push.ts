import { api } from "@/shared/api/client";
import { currentSubscription, saveSubscription } from "./browser-subscription";
import { pushSupport } from "./push-support";

/**
 * Ao sair da conta: tira a inscrição deste navegador da API e do serviço de
 * push, para os avisos da conta não continuarem chegando a quem usar o
 * navegador depois (nem passarem para a próxima conta que entrar, pelo
 * `PushLocaleSync`). Roda antes do logout, com a sessão ainda valendo; se
 * falhar, o logout segue do mesmo jeito.
 */
export async function forgetPush() {
  if (pushSupport() !== "supported") return;
  try {
    const subscription = await currentSubscription();
    if (subscription) {
      await api.DELETE("/push/subscriptions", {
        body: { endpoint: subscription.endpoint },
      });
      await subscription.unsubscribe();
    }
    saveSubscription(undefined);
  } catch {
    // Sem rede: a inscrição fica na API até o próximo envio dar 404/410 ou
    // outra conta entrar neste navegador.
  }
}
