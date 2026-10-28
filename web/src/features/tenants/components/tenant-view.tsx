import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { InviteSection } from "@/features/invites/components/invite-section";
import { LeaveTenantSection } from "@/features/members/components/leave-tenant-section";
import { MemberList } from "@/features/members/components/member-list";
import { NicknameForm } from "@/features/members/components/nickname-form";
import { TransferOwnershipSection } from "@/features/members/components/transfer-ownership-section";
import { TournamentList } from "@/features/tournaments/components/tournament-list";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";

/**
 * Página do tenant, só para membros: torneios, apelido, membros, (organizer
 * ou owner) convites e, por último, sair (ou, para o owner, transferir a
 * posse). Cada seção busca o próprio dado. Lê a sessão, então
 * fica em `<Suspense>`.
 */
export async function TenantView({ slug }: { slug: string }) {
  const api = await serverApi();
  const [t, tRoles, result] = await Promise.all([
    getTranslations("Tenant"),
    getTranslations("Roles"),
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
      <Alert>{await apiErrorMessage(result?.error?.error ?? "internal")}</Alert>
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
      <NicknameForm slug={slug} nickname={me.nickname} />
      <MemberList slug={slug} viewerRole={me.role} />
      {me.role !== "player" && <InviteSection slug={slug} />}
      {me.role === "owner" ? (
        <TransferOwnershipSection slug={slug} />
      ) : (
        <LeaveTenantSection slug={slug} />
      )}
    </>
  );
}
