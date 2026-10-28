import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { InviteSection } from "@/features/invites/components/invite-section";
import { MemberList } from "@/features/members/components/member-list";
import { MembershipSection } from "@/features/members/components/membership-section";
import { TournamentList } from "@/features/tournaments/components/tournament-list";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { PageError } from "@/shared/ui/page-error";

/**
 * Página do tenant, só para membros: torneios, membros, (organizer ou owner)
 * convites e, por último, o que é de quem vê (apelido e sair, ou, para o
 * owner, transferir a posse). Cada seção busca o próprio dado. Lê a sessão,
 * então fica em `<Suspense>`.
 */
export async function TenantView({ slug }: { slug: string }) {
  const api = await serverApi();
  const [t, tRoles, tNav, result] = await Promise.all([
    getTranslations("Tenant"),
    getTranslations("Roles"),
    getTranslations("Nav"),
    api
      .GET("/tenants/{slug}", { params: { path: { slug } } })
      .catch(() => undefined),
  ]);

  if (result?.response.status === 401) {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  const me = result?.data;
  if (!me) {
    return (
      <PageError href="/" label={tNav("backHome")}>
        {await apiErrorMessage(result?.error?.error ?? "internal")}
      </PageError>
    );
  }

  return (
    <>
      <header className="flex flex-col gap-1">
        <h1 className="text-3xl font-semibold">{me.name}</h1>
        <p className="text-sm opacity-70">
          {t("yourRole", { role: tRoles(me.role) })}
        </p>
      </header>
      <TournamentList slug={slug} viewerRole={me.role} />
      <MemberList slug={slug} viewerRole={me.role} />
      {me.role !== "player" && <InviteSection slug={slug} />}
      <MembershipSection
        slug={slug}
        nickname={me.nickname}
        owner={me.role === "owner"}
      />
    </>
  );
}
