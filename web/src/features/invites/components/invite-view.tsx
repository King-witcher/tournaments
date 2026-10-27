import { getFormatter, getTranslations } from "next-intl/server";

import { RequireLogin } from "@/features/auth/components/require-login";
import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";
import { AcceptInviteButton } from "./accept-invite-button";

/** Para qual comunidade é o convite; aceitar exige login. Fica em `<Suspense>`. */
export async function InviteView({ token }: { token: string }) {
  const api = await serverApi();
  const [t, format, preview] = await Promise.all([
    getTranslations("Invite"),
    getFormatter(),
    api
      .GET("/invites/{token}", { params: { path: { token } } })
      .catch(() => undefined),
  ]);

  const invite = preview?.data;
  if (!invite) {
    return (
      <Alert>
        {await apiErrorMessage(preview?.error?.error ?? "internal")}
      </Alert>
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
      <RequireLogin message={t("loginRequired")}>
        <AcceptInviteButton token={token} />
      </RequireLogin>
    </>
  );
}
