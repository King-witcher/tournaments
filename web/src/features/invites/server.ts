import type { Metadata, ResolvingMetadata } from "next";
import { headers } from "next/headers";
import type { Locale } from "next-intl";
import { getLocale, getTranslations } from "next-intl/server";
import { cache } from "react";

import { routing } from "@/i18n/routing";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { siteOpenGraph } from "@/shared/lib/link-preview";

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
 * imagem inteira, sem moldura. No link de inscrição, a descrição começa pela
 * comunidade e pelo modo, como no tile, e segue com a descrição do torneio, ou
 * as regras, ou a descrição da comunidade; no link da comunidade, é a descrição
 * dela. Nada que muda com o tempo (membros, vagas): a rede guarda a prévia na
 * primeira vez. No link de inscrição com capa, a capa (2:1) grande; senão o logo
 * da comunidade (1:1) como miniatura. Sem imagem nenhuma, a prévia do site.
 * Convite que não vale mais: a prévia do site inteira. Fica fora da busca, pela
 * regra do layout. O link vai sem idioma, e o robô que não diz o dele cai no
 * padrão do site: ali, o link de inscrição sai no idioma do torneio.
 */
export async function inviteMetadata(
  token: string,
  parent: ResolvingMetadata,
): Promise<Metadata> {
  const invite = (await getInvite(token))?.data;
  if (!invite) return {};
  const { tournament } = invite;
  const pageLocale = await getLocale();
  const locale =
    tournament &&
    pageLocale === routing.defaultLocale &&
    !(await headers()).has("accept-language")
      ? tournament.locale
      : pageLocale;
  const t = await getTranslations({ locale, namespace: "Invite" });
  const name = tournament && localizedName(tournament, locale);
  const title = name
    ? t("tournamentTitle", { name })
    : t("title", { name: invite.tenant_name });
  // `null`, e não `undefined`, para não herdar a do site.
  const description =
    shorten(
      tournament
        ? await tournamentDescription(
            tournament,
            invite.tenant_name,
            invite.tenant_description,
            locale,
          )
        : (invite.tenant_description ?? ""),
    ) || null;
  // Os tamanhos do corte feito no envio (docs/decisions/images.md).
  const cover = tournament?.cover_url;
  const logo = invite.tenant_logo_url;
  const images = cover
    ? [
        {
          url: cover,
          width: 1200,
          height: 600,
          type: "image/jpeg",
          alt: name ?? "",
        },
      ]
    : logo
      ? [
          {
            url: logo,
            width: 256,
            height: 256,
            type: "image/jpeg",
            alt: invite.tenant_name,
          },
        ]
      : ((await parent).openGraph?.images ?? []);
  // Os de Open Graph e do X trocam os do site inteiros: sem descrição, nada.
  const shared = { title, description: description ?? undefined, images };
  return {
    title,
    description,
    openGraph: { ...(await siteOpenGraph(locale)), ...shared },
    twitter: {
      ...shared,
      card: !cover && logo ? "summary" : "summary_large_image",
    },
  };
}

/** "Arena Sul · Duelo. A descrição…". */
async function tournamentDescription(
  tournament: InvitedTournament,
  tenantName: string,
  tenantDescription: string | null | undefined,
  locale: Locale,
) {
  const t = await getTranslations({ locale, namespace: "Tournaments" });
  const mode =
    tournament.mode === "tdm"
      ? t("modeTeams", { size: tournament.team_size })
      : t(`mode.${tournament.mode}`);
  const summary = `${tenantName} · ${mode}`;
  const text =
    localizedDescription(tournament, locale) ||
    localizedRules(tournament, locale) ||
    tenantDescription;
  return text ? `${summary}. ${text}` : summary;
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

function localizedRules(tournament: InvitedTournament, locale: string) {
  return (
    tournament.translations.find((tr) => tr.locale === locale)?.rules ||
    tournament.rules
  );
}

/** Numa linha só e cortada na última palavra que cabe, com reticências. */
function shorten(text: string) {
  const line = text.replace(/\s+/g, " ").trim();
  if (line.length <= PREVIEW_DESCRIPTION_MAX) return line;
  const cut = line.slice(0, PREVIEW_DESCRIPTION_MAX);
  return `${cut.slice(0, cut.lastIndexOf(" ")) || cut}…`;
}
