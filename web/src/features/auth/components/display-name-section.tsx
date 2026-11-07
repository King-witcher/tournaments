import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { getCurrentUser } from "../server";
import { DisplayNameForm } from "./display-name-form";

/** O nome exibido, no Perfil da conta: o que aparece no cabeçalho. Fica em `<Suspense>`. */
export async function DisplayNameSection() {
  const [user, t] = await Promise.all([
    getCurrentUser(),
    getTranslations("Account.name"),
  ]);
  if (!user) return null;

  return (
    <Section title={t("title")}>
      <DisplayNameForm name={user.display_name} />
    </Section>
  );
}
