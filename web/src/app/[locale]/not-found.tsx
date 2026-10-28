import { getTranslations } from "next-intl/server";

import { Page } from "@/shared/ui/page";
import { PageError } from "@/shared/ui/page-error";
import { PageHeader } from "@/shared/ui/page-header";

/** Endereço que não existe: o título, o que houve e o caminho para o início. */
export default async function NotFound() {
  const t = await getTranslations("NotFound");
  return (
    <Page>
      <PageHeader title={t("title")} />
      <PageError href="/" label={t("home")}>
        {t("text")}
      </PageError>
    </Page>
  );
}
