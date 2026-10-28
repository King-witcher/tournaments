import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { getCurrentUser } from "../server";
import { DisplayNameForm } from "./display-name-form";

/** A aba Perfil da conta: o nome que aparece no cabeçalho. Fica em `<Suspense>`. */
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
