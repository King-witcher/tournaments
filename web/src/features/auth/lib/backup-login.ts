import type { components } from "@/shared/api/schema";
import { PROVIDERS, type Provider } from "./providers";

/**
 * O provedor de quem depende só dele para entrar: sem email e com um único
 * provedor. Se ele ficar indisponível, a pessoa fica de fora. Com email, o
 * reserva já existe.
 */
export function soleProviderWithoutEmail(
  logins: components["schemas"]["LoginMethods"],
): Provider | undefined {
  if (logins.email) return undefined;
  const linked = PROVIDERS.filter((provider) => logins[provider]);
  return linked.length === 1 ? linked[0] : undefined;
}
