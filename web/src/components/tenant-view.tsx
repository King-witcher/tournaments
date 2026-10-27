import Image from "next/image";
import { getFormatter, getTranslations } from "next-intl/server";
import { apiErrorMessage } from "@/lib/api/errors";
import type { components } from "@/lib/api/schema";
import { serverApi } from "@/lib/api/server";
import { CreateInviteButton } from "./create-invite-button";
import { LoginPrompt } from "./login-prompt";
import { MemberRoleButton } from "./member-role-button";
import { NicknameForm } from "./nickname-form";
import { RevokeInviteButton } from "./revoke-invite-button";

type Invite = components["schemas"]["Invite"];

/**
 * Página do tenant, só para membros: apelido, membros e (organizer ou owner)
 * convites. Lê a sessão, então fica em `<Suspense>`.
 */
export async function TenantView({ slug }: { slug: string }) {
  const api = await serverApi();
  const path = { params: { path: { slug } } };
  const [t, tRoles, result] = await Promise.all([
    getTranslations("Tenant"),
    getTranslations("Roles"),
    api.GET("/tenants/{slug}", path).catch(() => undefined),
  ]);

  if (result?.response.status === 401) {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  const me = result?.data;
  if (!me) {
    return (
      <ErrorText
        message={await apiErrorMessage(result?.error?.error ?? "internal")}
      />
    );
  }

  const canInvite = me.role !== "player";
  const [members, invites] = await Promise.all([
    api.GET("/tenants/{slug}/members", path).then(({ data }) => data ?? []),
    canInvite
      ? api.GET("/tenants/{slug}/invites", path).then(({ data }) => data ?? [])
      : undefined,
  ]);

  return (
    <>
      <header className="flex flex-col gap-1">
        <h1 className="text-3xl font-semibold">{me.name}</h1>
        <p className="text-sm opacity-70">
          {t("yourRole", { role: tRoles(me.role) })}
        </p>
      </header>

      <NicknameForm slug={slug} nickname={me.nickname} />

      <section className="flex flex-col gap-3">
        <h2 className="text-xl font-semibold">
          {t("members.title", { count: members.length })}
        </h2>
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {members.map((m) => (
            <li key={m.user_id} className="flex items-center gap-3 py-2">
              {m.avatar_url ? (
                <Image
                  src={m.avatar_url}
                  alt=""
                  width={28}
                  height={28}
                  className="rounded-full"
                />
              ) : (
                <span className="size-7 rounded-full bg-black/10 dark:bg-white/10" />
              )}
              <span className="flex flex-1 flex-col">
                <span className="font-medium">{m.nickname}</span>
                {m.display_name !== m.nickname && (
                  <span className="text-xs opacity-60">{m.display_name}</span>
                )}
              </span>
              <span className="text-sm opacity-70">{tRoles(m.role)}</span>
              {me.role === "owner" && m.role !== "owner" && (
                <MemberRoleButton
                  slug={slug}
                  userId={m.user_id}
                  role={m.role}
                />
              )}
            </li>
          ))}
        </ul>
      </section>

      {invites && <InviteSection slug={slug} invites={invites} />}
    </>
  );
}

async function InviteSection({
  slug,
  invites,
}: {
  slug: string;
  invites: Invite[];
}) {
  const [t, format] = await Promise.all([
    getTranslations("Tenant.invites"),
    getFormatter(),
  ]);
  const now = new Date();
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <p className="text-sm opacity-70">{t("description")}</p>
      <CreateInviteButton slug={slug} />
      {invites.length === 0 ? (
        <p className="text-sm opacity-70">{t("empty")}</p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {invites.map((invite) => (
            <li key={invite.id} className="flex items-center gap-3 py-2">
              <span className="flex flex-1 flex-col text-sm">
                <span>
                  {t("item", {
                    name: invite.created_by,
                    expires: format.relativeTime(
                      new Date(invite.expires_at),
                      now,
                    ),
                  })}
                </span>
                <span className="text-xs opacity-60">
                  {t("uses", { count: invite.uses })}
                </span>
              </span>
              <RevokeInviteButton slug={slug} inviteId={invite.id} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function ErrorText({ message }: { message: string }) {
  return (
    <p
      role="alert"
      className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm"
    >
      {message}
    </p>
  );
}
