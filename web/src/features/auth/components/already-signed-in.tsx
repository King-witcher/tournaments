import { getTranslations } from "next-intl/server";

import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { getCurrentUser } from "../server";

/**
 * No login e no cadastro, para quem já entrou (voltou para a página pelo
 * histórico, ou abriu o link de novo): diz com que conta e leva de volta,
 * em vez de mostrar só o formulário. O formulário continua embaixo, para
 * entrar com outra conta. Fica em `<Suspense>`.
 */
export async function AlreadySignedIn({ returnTo }: { returnTo: string }) {
  const user = await getCurrentUser();
  if (!user) return null;
  const t = await getTranslations("Auth");

  return (
    <Notice
      kind="info"
      title={t("alreadySignedIn", { name: user.display_name })}
      action={<Link href={returnTo}>{t("continueSignedIn")}</Link>}
    />
  );
}
