import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { PageError } from "@/shared/ui/page-error";

/** Endereço que não existe: diz isso e leva ao início. */
export default async function NotFound() {
  const t = await getTranslations("NotFound");
  return (
    <Page>
      <PageError href="/" label={t("home")}>
        {t("text")}
      </PageError>
    </Page>
  );
}
