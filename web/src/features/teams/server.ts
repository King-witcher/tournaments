import { getTranslations } from "next-intl/server";
import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { teamName } from "./lib/team-name";

/**
 * Como `useTeamName`, para Server Components assíncronos (que não usam
 * hooks): a função que dá o nome exibido de um time no idioma de quem vê.
 */
export async function getTeamName() {
  const [tTeam, tMember] = await Promise.all([
    getTranslations("Team"),
    getTranslations("Member"),
  ]);
  return (team: components["schemas"]["Team"]) =>
    teamName(team, {
      deleted: tMember("deletedAccount"),
      teamOf: (captain) => tTeam("teamOf", { captain }),
    });
}

/**
 * A formação dos times (os times, os reservas e quem fez check-in), uma
 * busca por requisição: a página dos times, a aba Inscritos, o resumo da aba
 * Organização, o "Iniciar" e o seeding dividem o resultado. `undefined` se a
 * API falhar. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getFormation = cache(async (tenant: string, slug: string) => {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/tournaments/{tournament}/teams", {
      params: { path: { slug: tenant, tournament: slug } },
    })
    .then(({ data }) => data)
    .catch(() => undefined);
});

/**
 * Os times elegíveis na ordem que vale ao iniciar (seed, depois quem não
 * tem), para o resumo da aba Organização e a página do seeding; `undefined`
 * se a API falhar. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getSeeding = cache(async (tenant: string, slug: string) => {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/tournaments/{tournament}/seeding", {
      params: { path: { slug: tenant, tournament: slug } },
    })
    .then(({ data }) => data)
    .catch(() => undefined);
});
