import { useTranslations } from "next-intl";

import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { PageHeader } from "@/shared/ui/page-header";

/** A sala venceu ou não existe, com o caminho para criar outra. */
export function VetoRoomGone() {
  const t = useTranslations("VetoRoom");
  return (
    <>
      <PageHeader title={t("title")} />
      <EmptyState
        action={<ButtonLink href="/veto">{t("gone.create")}</ButtonLink>}
      >
        {t("gone.text")}
      </EmptyState>
    </>
  );
}
