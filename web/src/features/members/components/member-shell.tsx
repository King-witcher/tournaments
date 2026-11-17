import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { getCurrentUser } from "@/features/auth/server";
import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Avatar } from "@/shared/ui/avatar";
import { LocalDate } from "@/shared/ui/local-date";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { PageTabs } from "@/shared/ui/page-tabs";
import { getMember } from "../server";
import { EditNicknameButton } from "./edit-nickname-button";
import { LeaveTenantMenu } from "./leave-tenant-menu";
import styles from "./member-shell.module.scss";

type Props = {
  tenant: string;
  /** O `user_id` do endereço, ainda como texto. */
  user: string;
  /** A aba atual (o `children` do `layout.tsx`). */
  children: ReactNode;
};

/**
 * A casca da página da pessoa, só para membros: o cabeçalho (avatar,
 * apelido, papel e desde quando é membro; se for você, "Editar apelido" e,
 * no ⋯, "Sair da comunidade", que o owner não tem: transfere a posse antes)
 * e as abas Visão geral, Partidas, Torneios e Confrontos, com a atual
 * embaixo. A própria página fica no menu lateral ("Seu perfil"): a trilha
 * segue a das seções (só abaixo do `lg`); a de outra pessoa leva a Membros.
 * Quem não é (ou não é mais) membro: `member.not_found`, sem as abas. Fica
 * em `<Suspense>`.
 */
export async function MemberShell({ tenant, user, children }: Props) {
  const [loaded, result, me, t, tRoles, tNav] = await Promise.all([
    getTenant(tenant),
    getMember(tenant, user),
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
  if (!result.ok) {
    return (
      <PageError href={`/${tenant}/members`} label={t("backToMembers")}>
        {await apiErrorMessage(result.code)}
      </PageError>
    );
  }

  const profile = result.data;
  const isYou = me?.id === profile.user_id;
  const base = `/${tenant}/members/${profile.user_id}`;
  const community = { href: `/${tenant}`, label: loaded.tenant.name };
  return (
    <>
      <PageHeader
        breadcrumbs={
          isYou
            ? [community]
            : [community, { href: `/${tenant}/members`, label: t("members") }]
        }
        inSideNav={isYou}
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
        meta={
          <>
            {tRoles(profile.role)}
            {" · "}
            {t.rich("since", {
              date: () => <LocalDate value={profile.joined_at} />,
            })}
            {profile.display_name !== profile.nickname &&
              ` · ${profile.display_name}`}
          </>
        }
        actions={
          isYou && (
            <EditNicknameButton slug={tenant} nickname={profile.nickname} />
          )
        }
        overflow={
          isYou && profile.role !== "owner" && <LeaveTenantMenu slug={tenant} />
        }
      />
      <PageTabs
        label={t("tabs.label")}
        tabs={[
          { href: base, label: t("tabs.overview") },
          { href: `${base}/games`, label: t("tabs.games") },
          { href: `${base}/tournaments`, label: t("tabs.tournaments") },
          { href: `${base}/opponents`, label: t("tabs.opponents") },
        ]}
      >
        {children}
      </PageTabs>
    </>
  );
}
