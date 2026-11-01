import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { ImageResponse } from "next/og";
import { hasLocale } from "next-intl";

import { InviteCard } from "@/features/invites/lib/invite-card";
import { routing } from "@/i18n/routing";
import type { components } from "@/shared/api/schema";
import { API_URL } from "@/shared/api/url";
import { BrandCard } from "@/shared/lib/brand-card";
import esInvite from "../../../../../messages/es/Invite.json";
import esLanding from "../../../../../messages/es/Landing.json";
import ptInvite from "../../../../../messages/pt/Invite.json";
import ptLanding from "../../../../../messages/pt/Landing.json";

export const size = { width: 1200, height: 630 };
export const contentType = "image/png";
export const alt = "Telefrag";

// Os textos vêm direto dos JSON, como na prévia do site
// (`[locale]/opengraph-image.tsx`): uma rota de imagem não tem o idioma em
// `next/root-params`.
const TEXTS = {
  pt: { invite: ptInvite, landing: ptLanding },
  es: { invite: esInvite, landing: esLanding },
};

const plex = (weight: string) =>
  readFile(
    join(
      process.cwd(),
      `node_modules/@ibm/plex-sans/fonts/complete/woff/IBMPlexSans-${weight}.woff`,
    ),
  );

type Preview = components["schemas"]["InvitePreview"];

/** A prévia pública do convite; `undefined` se ele não vale mais. */
async function loadInvite(token: string): Promise<Preview | undefined> {
  const res = await fetch(`${API_URL}/invites/${encodeURIComponent(token)}`, {
    headers: { accept: "application/json" },
  }).catch(() => undefined);
  return res?.ok ? ((await res.json()) as Preview) : undefined;
}

/**
 * A imagem do CDN embutida (`data:`): o `ImageResponse` que busca sozinho
 * falha a prévia inteira quando uma imagem não carrega (e recusa endereço
 * local, o do bucket de dev). Sem a imagem, a prévia sai sem ela.
 */
async function embed(url: string | null | undefined) {
  if (!url) return undefined;
  const res = await fetch(url).catch(() => undefined);
  const type = res?.headers.get("content-type") ?? "";
  if (!res?.ok || !type.startsWith("image/")) return undefined;
  const bytes = Buffer.from(await res.arrayBuffer());
  return `data:${type};base64,${bytes.toString("base64")}`;
}

/**
 * A prévia do link de convite (Discord, WhatsApp): o logo e o nome da
 * comunidade e, no link de inscrição, o nome e a capa do torneio. O convite
 * é público (quem tem o link vê a prévia na página), então a prévia não
 * mostra nada além disso. Convite que não vale mais: a prévia do site.
 */
export default async function OpengraphImage({
  params,
}: {
  params: Promise<{ locale: string; token: string }>;
}) {
  const { locale: requested, token } = await params;
  const locale = hasLocale(routing.locales, requested)
    ? requested
    : routing.defaultLocale;
  const [regular, semibold, invite] = await Promise.all([
    plex("Regular"),
    plex("SemiBold"),
    loadInvite(token),
  ]);
  const texts = TEXTS[locale];
  const tournament = invite?.tournament;
  const [logo, cover] = await Promise.all([
    embed(invite?.tenant_logo_url),
    embed(tournament?.cover_url),
  ]);
  const name =
    tournament &&
    (tournament.translations.find((tr) => tr.locale === locale)?.name ??
      tournament.name);
  const card = invite ? (
    <InviteCard
      {...size}
      title={(tournament
        ? texts.invite.tournamentTitle
        : texts.invite.title
      ).replace("{name}", name ?? invite.tenant_name)}
      community={invite.tenant_name}
      logo={logo}
      cover={cover}
    />
  ) : (
    <BrandCard
      {...size}
      title={texts.landing.hero.title}
      description={texts.landing.hero.lead}
    />
  );
  return new ImageResponse(card, {
    ...size,
    fonts: [
      { name: "IBM Plex Sans", data: regular, weight: 400 },
      { name: "IBM Plex Sans", data: semibold, weight: 600 },
    ],
  });
}
