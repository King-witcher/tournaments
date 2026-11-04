import type { Metadata, ResolvingMetadata } from "next";
import { getLocale, getTranslations } from "next-intl/server";
import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

type InvitedTournament = components["schemas"]["InvitedTournament"];

/**
 * A prévia pública do convite, uma busca por requisição: a página e a prévia
 * do link (`inviteMetadata`) dividem o resultado. `undefined` com a API fora.
 * Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getInvite = cache(async (token: string) => {
  const api = await serverApi();
  return api
    .GET("/invites/{token}", { params: { path: { token } } })
    .catch(() => undefined);
});

/** Até onde a descrição vai na prévia (o Discord mostra ~350). */
const PREVIEW_DESCRIPTION_MAX = 300;

/**
 * A prévia do link de convite (Discord, WhatsApp): o título do convite e a
 * imagem inteira, sem moldura, e a descrição da comunidade (no link de
 * inscrição, a do torneio, se houver). Nada que muda com o tempo (membros,
 * vagas): a rede guarda a prévia na primeira vez. No link de inscrição com
 * capa, a capa (2:1) grande; senão o logo da comunidade (1:1) como miniatura,
 * que o WhatsApp não corta. Sem imagem nenhuma, a prévia do site. Convite que
 * não vale mais: a prévia do site inteira.
 */
export async function inviteMetadata(
  token: string,
  parent: ResolvingMetadata,
): Promise<Metadata> {
  const invite = (await getInvite(token))?.data;
  if (!invite) return {};
  const [t, tNav, locale] = await Promise.all([
    getTranslations("Invite"),
    getTranslations("Nav"),
    getLocale(),
  ]);
  const { tournament } = invite;
  const title = tournament
    ? t("tournamentTitle", { name: localizedName(tournament, locale) })
    : t("title", { name: invite.tenant_name });
  // `null`, e não `undefined`, para não herdar a do site.
  const description =
    shorten(
      (tournament && localizedDescription(tournament, locale)) ||
        (invite.tenant_description ?? ""),
    ) || null;
  // Os tamanhos do corte feito no envio (docs/decisions/images.md).
  const cover = tournament?.cover_url;
  const logo = invite.tenant_logo_url;
  const images = cover
    ? [{ url: cover, width: 1200, height: 600 }]
    : logo
      ? [{ url: logo, width: 256, height: 256 }]
      : ((await parent).openGraph?.images ?? []);
  // Os de Open Graph e do X trocam os do site inteiros: sem descrição, nada.
  const shared = { title, description: description ?? undefined, images };
  return {
    title,
    description,
    openGraph: { ...shared, siteName: tNav("brand") },
    twitter: {
      ...shared,
      card: !cover && logo ? "summary" : "summary_large_image",
    },
  };
}

/**
 * Nome no idioma de quem vê, como o `localizedName` de `tournaments` (que
 * não é importado: aquela feature já monta os links de inscrição desta).
 */
export function localizedName(tournament: InvitedTournament, locale: string) {
  return (
    tournament.translations.find((tr) => tr.locale === locale)?.name ??
    tournament.name
  );
}

export function localizedDescription(
  tournament: InvitedTournament,
  locale: string,
) {
  return (
    tournament.translations.find((tr) => tr.locale === locale)?.description ||
    tournament.description
  );
}

/** Numa linha só e cortada na última palavra que cabe, com reticências. */
function shorten(text: string) {
  const line = text.replace(/\s+/g, " ").trim();
  if (line.length <= PREVIEW_DESCRIPTION_MAX) return line;
  const cut = line.slice(0, PREVIEW_DESCRIPTION_MAX);
  return `${cut.slice(0, cut.lastIndexOf(" ")) || cut}…`;
}
