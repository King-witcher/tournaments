import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { apiErrorMessage } from "@/shared/api/errors";
import { ButtonLink } from "@/shared/ui/button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";
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
 * para a organização; o Ranking, para todos. Com a exclusão agendada, o aviso
 * com a data (e, para o dono, onde cancelar) entre o topo e as abas. Sem
 * sessão, o convite a entrar; sem acesso, o erro (e a aba nem é desenhada).
 * Fica em `<Suspense>`.
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
      {tenant.deletion_scheduled_for && (
        <DeletionNotice
          date={tenant.deletion_scheduled_for}
          owner={tenant.role === "owner"}
        />
      )}
      <PageTabs label={t("sections")} tabs={tabs}>
        {children}
      </PageTabs>
    </>
  );
}

type DeletionNoticeProps = {
  /** Quando a comunidade vai ser excluída (ISO). */
  date: string;
  /** Quem vê é o dono, que cancela na aba Configurações. */
  owner: boolean;
};

/**
 * A comunidade vai ser excluída e, até lá, é só leitura. Sem link para o
 * dono: o `InlineNotification` do Carbon não aceita controle dentro dele, e
 * a aba Configurações está logo embaixo.
 */
async function DeletionNotice({ date, owner }: DeletionNoticeProps) {
  const t = await getTranslations("Tenant.deletionNotice");
  return (
    <Notice kind="warning" title={t("title")}>
      {/* Num span: o conteúdo do aviso é flex, e a data quebraria a frase. */}
      <span>
        {t.rich("text", { date: () => <LocalDateTime value={date} /> })}
        {owner && ` ${t("ownerHint")}`}
      </span>
    </Notice>
  );
}
