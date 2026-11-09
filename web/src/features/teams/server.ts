import { getTranslations } from "next-intl/server";
import { cache } from "react";
import { hasStarted, seedingOpen } from "@/features/tournaments/lib/status";
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
 * busca por requisição: a página dos times, a aba Inscritos, o "Iniciar" e
 * o seeding dividem o resultado. `undefined` se a
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
 * tem), para o resumo em Inscritos e a página do seeding; `undefined`
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

/**
 * O resumo da ordem da chave em Inscritos, para a organização, até iniciar:
 * uma frase com quantos entram e como está a ordem (fora da revisão, quando
 * ela vale) e se já dá para defini-la. Depois de iniciar, `undefined`: a
 * ordem usada está na lista. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export async function getSeedingSummary(
  tenant: string,
  tournament: components["schemas"]["Tournament"],
): Promise<{ text?: string; editable: boolean } | undefined> {
  if (hasStarted(tournament.status)) return undefined;
  const t = await getTranslations("Participants.seeding");
  const hasCheckIn = tournament.check_in;
  if (!seedingOpen(tournament)) {
    const key = hasCheckIn ? "summary.atCheckIn" : "summary.atRegistration";
    return { text: t(key), editable: false };
  }

  const order = await getSeeding(tenant, tournament.slug);
  if (!order) return { text: t("summary.unavailable"), editable: false };
  // No TDM, a ordem é de times (os completos), não de inscritos.
  const teams = tournament.mode === "tdm";
  if (order.length === 0) {
    // Sem check-in, ninguém elegível é ninguém inscrito: a tabela vazia já diz.
    if (teams) return { text: t("summary.emptyTeams"), editable: false };
    return { text: hasCheckIn ? t("empty") : undefined, editable: false };
  }
  const count = order.length;
  const entering = teams
    ? t("summary.countTeams", { count })
    : t("summary.count", { count, total: tournament.participants });
  const state = order.every((p) => p.seed == null)
    ? t("summary.random")
    : order.some((p) => p.seed == null)
      ? t("summary.partial")
      : t("summary.set");
  return { text: `${entering} ${state}`, editable: true };
}
