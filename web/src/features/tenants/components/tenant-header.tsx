import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { apiErrorMessage } from "@/shared/api/errors";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";
import { TENANT_TITLE_ID } from "../lib/focus";
import { getTenant } from "../server";
import { TenantDeletionNotice } from "./tenant-deletion-notice";

type Props = {
  slug: string;
  /** O que a seção é (Torneios, Ranking, Membros, Configurações). */
  title: string;
  /** Metadado curto da seção (o número de membros). */
  meta?: ReactNode;
  /**
   * A ação primária da seção; cada uma se esconde sozinha de quem não pode
   * usá-la (`NewTournamentAction`, `MemberInviteAction`).
   */
  actions?: ReactNode;
  /** O início da comunidade (Torneios): sem a trilha, que levaria a ela mesma. */
  home?: boolean;
  /** A seção é a das Configurações: o aviso da exclusão não leva até ela. */
  settings?: boolean;
};

/**
 * O topo de cada seção da comunidade: a trilha (a comunidade, menos no
 * início dela), o título da seção, o metadado e a ação primária. O nome, o logo e o papel de
 * quem vê ficam no menu lateral; no celular, sem o menu à vista, a trilha
 * mostra a comunidade. Com a exclusão agendada, o aviso logo abaixo, em toda
 * seção. Sem sessão, o convite a entrar; sem acesso, o erro (o conteúdo das
 * seções não aparece: cada bloco confere o acesso e some). Fica em
 * `<Suspense>`.
 */
export async function TenantHeader({
  slug,
  title,
  meta,
  actions,
  home = false,
  settings = false,
}: Props) {
  const [loaded, t, tNav] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant"),
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
  return (
    <>
      <PageHeader
        breadcrumbs={
          home ? undefined : [{ href: `/${slug}`, label: tenant.name }]
        }
        // tabIndex -1: recebe o foco depois de criar a comunidade.
        title={
          <span id={TENANT_TITLE_ID} tabIndex={-1}>
            {title}
          </span>
        }
        meta={meta}
        actions={actions}
      />
      {tenant.deletion_scheduled_for && (
        <TenantDeletionNotice
          slug={slug}
          date={tenant.deletion_scheduled_for}
          cancel={tenant.role === "owner" && !settings}
        />
      )}
    </>
  );
}
