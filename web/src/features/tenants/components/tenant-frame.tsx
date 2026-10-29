import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { apiErrorMessage } from "@/shared/api/errors";
import { ButtonLink } from "@/shared/ui/button-link";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { PageTabs } from "@/shared/ui/page-tabs";
import { StatusTag } from "@/shared/ui/status-tag";
import { getTenant } from "../server";

type Props = {
  slug: string;
  /** A aba aberta (a sub-rota do layout). */
  children: ReactNode;
};

/**
 * O topo da comunidade e as abas, com a aba aberta dentro: nome, o papel de
 * quem vê e, para a organização, "Novo torneio". A aba Convites só aparece
 * para a organização; o Ranking, para todos. Sem sessão, o convite a entrar; sem acesso, o erro (e
 * a aba nem é desenhada). Fica em `<Suspense>`.
 */
export async function TenantFrame({ slug, children }: Props) {
  const [loaded, t, tRoles, tNav] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant"),
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

  const { tenant } = loaded;
  const organizer = tenant.role !== "player";
  const tabs = [
    { href: `/${slug}`, label: t("tabs.tournaments") },
    { href: `/${slug}/ranking`, label: t("tabs.ranking") },
    { href: `/${slug}/members`, label: t("tabs.members") },
    ...(organizer
      ? [{ href: `/${slug}/invites`, label: t("tabs.invites") }]
      : []),
    { href: `/${slug}/settings`, label: t("tabs.settings") },
  ];

  return (
    <>
      <PageHeader
        breadcrumbs={[{ href: "/", label: tNav("home") }]}
        title={tenant.name}
        tags={<StatusTag status="neutral">{tRoles(tenant.role)}</StatusTag>}
        actions={
          organizer && (
            <ButtonLink href={`/${slug}/new`}>{t("newTournament")}</ButtonLink>
          )
        }
      />
      <PageTabs label={t("sections")} tabs={tabs}>
        {children}
      </PageTabs>
    </>
  );
}
