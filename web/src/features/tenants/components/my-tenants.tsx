import { getTranslations } from "next-intl/server";

import { Link } from "@/i18n/navigation";
import { buttonVariants } from "@/shared/ui/button";
import { getMyTenants } from "../server";

/** Comunidades do usuário logado; sem login não mostra nada. Fica em `<Suspense>`. */
export async function MyTenants() {
  const tenants = await getMyTenants();
  if (!tenants) return null;

  const [t, tRoles] = await Promise.all([
    getTranslations("Home.tenants"),
    getTranslations("Roles"),
  ]);
  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-4">
        <h2 className="text-xl font-semibold">{t("title")}</h2>
        <Link
          href="/new"
          className={buttonVariants({ variant: "outline", size: "sm" })}
        >
          {t("create")}
        </Link>
      </div>
      {tenants.length === 0 ? (
        <p className="text-sm opacity-70">{t("empty")}</p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {tenants.map((tenant) => (
            <li key={tenant.slug}>
              <Link
                href={`/${tenant.slug}`}
                className="flex items-center justify-between gap-4 py-2 hover:underline"
              >
                <span className="font-medium">{tenant.name}</span>
                <span className="text-sm opacity-70">
                  {tRoles(tenant.role)}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
