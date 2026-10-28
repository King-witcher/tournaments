import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { getCurrentUser } from "@/features/auth/server";
import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { Avatar } from "@/shared/ui/avatar";
import { ButtonLink } from "@/shared/ui/button-link";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { StatusTag } from "@/shared/ui/status-tag";
import styles from "./member-profile.module.scss";
import { MemberSince } from "./member-since";
import { MemberTournaments } from "./member-tournaments";

type Props = {
  tenant: string;
  /** O `user_id` do endereço, ainda como texto. */
  user: string;
};

/** Um id de usuário válido no endereço, ou `undefined`. */
function parseUserId(user: string) {
  const id = Number(user);
  return /^\d+$/.test(user) && Number.isSafeInteger(id) && id > 0
    ? id
    : undefined;
}

/**
 * A pessoa na comunidade, só para membros: avatar, apelido, papel e desde
 * quando é membro (e, se for você, "Editar apelido"), e os torneios daqui em
 * que se inscreveu. Quem não é (ou não é mais) membro: `member.not_found`.
 * Fica em `<Suspense>`.
 */
export async function MemberProfile({ tenant, user }: Props) {
  const userId = parseUserId(user);
  const api = await serverApi();
  const [loaded, result, me, t, tRoles, tNav] = await Promise.all([
    getTenant(tenant),
    userId === undefined
      ? undefined
      : api
          .GET("/tenants/{slug}/members/{user_id}", {
            params: { path: { slug: tenant, user_id: userId } },
          })
          .catch(() => undefined),
    getCurrentUser(),
    getTranslations("Member"),
    getTranslations("Roles"),
    getTranslations("Nav"),
  ]);

  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return (
      <PageError href="/" label={tNav("backHome")}>
        {await apiErrorMessage(loaded.code)}
      </PageError>
    );
  }
  const profile = result?.data;
  if (!profile) {
    const code =
      userId === undefined
        ? "member.not_found"
        : (result?.error?.error ?? "internal");
    return (
      <PageError href={`/${tenant}/members`} label={t("backToMembers")}>
        {await apiErrorMessage(code)}
      </PageError>
    );
  }

  const isYou = me?.id === profile.user_id;
  return (
    <>
      <PageHeader
        breadcrumbs={[
          { href: "/", label: tNav("home") },
          { href: `/${tenant}`, label: loaded.tenant.name },
          { href: `/${tenant}/members`, label: t("members") },
        ]}
        title={
          <span className={styles.title}>
            <Avatar
              src={profile.avatar_url}
              name={profile.nickname}
              size="lg"
            />
            {profile.nickname}
          </span>
        }
        tags={<StatusTag status="neutral">{tRoles(profile.role)}</StatusTag>}
        meta={
          <>
            {t.rich("since", {
              date: () => <MemberSince value={profile.joined_at} />,
            })}
            {profile.display_name !== profile.nickname &&
              ` · ${profile.display_name}`}
          </>
        }
        actions={
          isYou && (
            <ButtonLink href={`/${tenant}/settings`} kind="tertiary">
              {t("editNickname")}
            </ButtonLink>
          )
        }
      />
      <MemberTournaments tenant={tenant} profile={profile} isYou={isYou} />
    </>
  );
}
