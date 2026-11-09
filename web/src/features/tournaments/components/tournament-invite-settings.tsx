import { getTranslations } from "next-intl/server";

import { InviteLinkSettings } from "@/features/invites/components/invite-link-settings";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { invitesAllowed } from "../lib/status";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** A comunidade vai ser excluída: ninguém mais entra. */
  deletionScheduled: boolean;
};

/**
 * Ao lado do formulário de Início e inscrições: o link de inscrição, o código,
 * "Gerar novo link" e, com as vagas lotadas, o aviso de que ele não inscreve
 * agora. Some fora de `invitesAllowed` e sem o token (a API falhou).
 */
export async function TournamentInviteSettings({
  tenant,
  tournament,
  deletionScheduled,
}: Props) {
  if (!invitesAllowed(tournament.status) || deletionScheduled) return null;
  const api = await serverApi();
  const [t, token] = await Promise.all([
    getTranslations("Invite.tournamentLink"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/invite", {
        params: { path: { slug: tenant, tournament: tournament.slug } },
      })
      .then(({ data }) => data?.token)
      .catch(() => undefined),
  ]);
  if (!token) return null;
  const full = tournament.participants >= tournament.max_participants;

  return (
    <Section level={3} title={t("title")} description={t("note")}>
      <Stack gap={5}>
        {full && <Notice kind="warning">{t("full")}</Notice>}
        <InviteLinkSettings
          tenant={tenant}
          tournament={tournament.slug}
          token={token}
          linkLabel={t("title")}
          rotateConfirm={t("rotateConfirm")}
        />
      </Stack>
    </Section>
  );
}
