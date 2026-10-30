import { getTranslations } from "next-intl/server";

import { ErrorPage } from "@/features/errors/components/error-page";

/** Endereço que não existe: o mesmo desenho do erro de execução, sem "Tentar de novo". */
export default async function NotFound() {
  const t = await getTranslations("NotFound");
  return <ErrorPage title={t("title")}>{t("text")}</ErrorPage>;
}
