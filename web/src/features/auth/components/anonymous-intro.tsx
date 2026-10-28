import { getTranslations } from "next-intl/server";

import { Link } from "@/i18n/navigation";
import { buttonVariants } from "@/shared/ui/button";
import { getCurrentUser } from "../server";

/**
 * Na home, para quem não entrou: o que fazer para participar, com entrar e
 * criar conta à vista. Logado, não mostra nada. Fica em `<Suspense>`.
 */
export async function AnonymousIntro() {
  const user = await getCurrentUser();
  if (user) return null;
  const t = await getTranslations("Home.anonymous");

  return (
    <section className="flex flex-col gap-3">
      <p className="text-sm">{t("hint")}</p>
      <div className="flex flex-wrap gap-3">
        <Link href="/login" className={buttonVariants()}>
          {t("login")}
        </Link>
        <Link href="/signup" className={buttonVariants({ variant: "outline" })}>
          {t("signup")}
        </Link>
      </div>
    </section>
  );
}
