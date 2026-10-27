import { getFormatter, getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { AcceptInviteButton } from "@/components/accept-invite-button";
import { LoginPrompt } from "@/components/login-prompt";
import { apiErrorMessage } from "@/lib/api/errors";
import { serverApi } from "@/lib/api/server";

export default async function InvitePage({
  params,
}: PageProps<"/[locale]/invite/[token]">) {
  const t = await getTranslations("Invite");
  return (
    <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-4 px-4 py-16">
      <Suspense fallback={<p>{t("loading")}</p>}>
        {params.then(({ token }) => (
          <InviteView token={token} />
        ))}
      </Suspense>
    </main>
  );
}

/** Mostra para qual comunidade é o convite; aceitar exige login. */
async function InviteView({ token }: { token: string }) {
  const api = await serverApi();
  const [t, format, preview, user] = await Promise.all([
    getTranslations("Invite"),
    getFormatter(),
    api
      .GET("/invites/{token}", { params: { path: { token } } })
      .catch(() => undefined),
    api
      .GET("/me")
      .then(({ data }) => data)
      .catch(() => undefined),
  ]);

  const invite = preview?.data;
  if (!invite) {
    return (
      <p
        role="alert"
        className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm"
      >
        {await apiErrorMessage(preview?.error?.error ?? "internal")}
      </p>
    );
  }

  return (
    <>
      <h1 className="text-3xl font-semibold">
        {t("title", { name: invite.tenant_name })}
      </h1>
      <p className="text-sm opacity-70">
        {t("expires", {
          expires: format.relativeTime(new Date(invite.expires_at), new Date()),
        })}
      </p>
      {user ? (
        <AcceptInviteButton token={token} />
      ) : (
        <LoginPrompt message={t("loginRequired")} />
      )}
    </>
  );
}
