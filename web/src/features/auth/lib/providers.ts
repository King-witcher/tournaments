/** Os provedores externos de login, com o nome das rotas na API. */
export const PROVIDERS = ["discord", "google"] as const;

export type Provider = (typeof PROVIDERS)[number];
