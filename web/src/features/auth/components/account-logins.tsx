import { apiErrorMessage } from "@/shared/api/errors";
import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";
import { DiscordLoginSection } from "./discord-login-section";
import { PasswordLoginSection } from "./password-login-section";

/** Formas de login da conta logada. Fica em `<Suspense>`. */
export async function AccountLogins() {
  const api = await serverApi();
  const result = await api.GET("/auth/logins").catch(() => undefined);
  const logins = result?.data;
  if (!logins) {
    return (
      <Alert>{await apiErrorMessage(result?.error?.error ?? "internal")}</Alert>
    );
  }

  return (
    <>
      <DiscordLoginSection
        linked={logins.discord}
        hasPassword={Boolean(logins.username)}
      />
      <PasswordLoginSection
        username={logins.username ?? undefined}
        discordLinked={logins.discord}
      />
    </>
  );
}
