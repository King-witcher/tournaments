import { useTranslations } from "next-intl";

import { ButtonLink } from "@/shared/ui/button-link";

/** "Definir a ordem", ao lado do título em Inscritos, enquanto ela se revisa. */
export function SeedingLink({
  tenant,
  slug,
}: {
  tenant: string;
  slug: string;
}) {
  const t = useTranslations("Participants.seeding");
  return (
    <ButtonLink href={`/${tenant}/${slug}/seeding`} kind="tertiary" size="sm">
      {t("edit")}
    </ButtonLink>
  );
}
