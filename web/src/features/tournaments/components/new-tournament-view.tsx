import { getTranslations } from "next-intl/server";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { Link } from "@/i18n/navigation";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";
import { TournamentForm } from "./tournament-form";

/** Criar torneio no tenant, para organizer e owner. Lê a sessão, então fica em `<Suspense>`. */
export async function NewTournamentView({ tenant }: { tenant: string }) {
  const api = await serverApi();
  const [t, result] = await Promise.all([
    getTranslations("NewTournament"),
    api
      .GET("/tenants/{slug}", { params: { path: { slug: tenant } } })
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
        <Link
          href={`/${tenant}`}
          className="text-sm text-muted underline underline-offset-4 hover:decoration-2"
        >
          {me.name}
        </Link>
        <h1 className="text-3xl font-semibold">{t("title")}</h1>
      </header>
      {me.role === "player" ? (
        <Alert>{await apiErrorMessage("tenant.forbidden")}</Alert>
      ) : (
        <TournamentForm tenant={tenant} />
      )}
    </>
  );
}
