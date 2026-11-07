import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";

/**
 * A entrada da sala de veto no início de quem entrou: uma ação do cabeçalho
 * (`PageHeader` `actions`), secundária, porque o início é das tarefas e dos
 * torneios. Leva à criação da sala.
 */
export async function VetoRoomEntry() {
  const t = await getTranslations("VetoRoom");
  return (
    <ButtonLink href="/veto" kind="tertiary">
      {t("create")}
    </ButtonLink>
  );
}
