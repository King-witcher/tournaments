import { getTranslations } from "next-intl/server";

import { Link } from "@/i18n/navigation";
import { getCurrentUser } from "../server";

/**
 * No login e no cadastro, para quem já entrou (voltou para a página pelo
 * histórico, ou abriu o link de novo): diz com que conta e leva de volta,
 * em vez de mostrar só o formulário. O formulário continua embaixo, para
 * entrar com outra conta.
 */
export async function AlreadySignedIn({ returnTo }: { returnTo: string }) {
  const user = await getCurrentUser();
  if (!user) return null;
  const t = await getTranslations("Auth");

  return (
    <p className="text-sm">
      {t("alreadySignedIn", { name: user.display_name })}{" "}
      <Link
        href={returnTo}
        className="inline-flex min-h-11 items-center underline underline-offset-4 hover:decoration-2 sm:min-h-0"
      >
        {t("continueSignedIn")}
      </Link>
    </p>
  );
}
