import { getTranslations } from "next-intl/server";
import { Suspense } from "react";
import { CreateTenantForm } from "@/components/create-tenant-form";
import { LoginPrompt } from "@/components/login-prompt";
import { serverApi } from "@/lib/api/server";

export default async function NewTenantPage() {
  const t = await getTranslations("NewTenant");
  return (
    <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col gap-4 px-4 py-16">
      <h1 className="text-3xl font-semibold">{t("title")}</h1>
      <Suspense fallback={null}>
        <NewTenant />
      </Suspense>
    </main>
  );
}

/** Formulário para quem está logado; os outros veem o botão de entrar. */
async function NewTenant() {
  const api = await serverApi();
  const [t, user] = await Promise.all([
    getTranslations("NewTenant"),
    api
      .GET("/me")
      .then(({ data }) => data)
      .catch(() => undefined),
  ]);
  if (!user) return <LoginPrompt message={t("loginRequired")} />;
  return <CreateTenantForm />;
}
