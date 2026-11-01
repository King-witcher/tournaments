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

/**
 * A prévia do link de convite (Discord, WhatsApp): o título do convite, a
 * descrição e a imagem inteira, sem moldura. No link de inscrição com capa,
 * a capa (2:1) grande; senão o logo da comunidade (1:1) como miniatura, que
 * o WhatsApp não corta. Sem imagem nenhuma, a prévia do site. Convite que
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
  const description = tournament
    ? t("tournamentPreview", {
        tenant: invite.tenant_name,
        spots: t("tournamentSpots", {
          count: tournament.participants,
          max: tournament.max_participants,
        }),
      })
    : t("preview", { count: invite.member_count, owner: invite.owner_name });
  // Os tamanhos do corte feito no envio (docs/decisions/images.md).
  const cover = tournament?.cover_url;
  const logo = invite.tenant_logo_url;
  const images = cover
    ? [{ url: cover, width: 1200, height: 600 }]
    : logo
      ? [{ url: logo, width: 256, height: 256 }]
      : ((await parent).openGraph?.images ?? []);
  return {
    title,
    description,
    openGraph: { title, description, siteName: tNav("brand"), images },
    twitter: {
      card: !cover && logo ? "summary" : "summary_large_image",
      title,
      description,
      images,
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
