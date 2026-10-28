import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "../server";
import { DisplayNameForm } from "./display-name-form";

/** O nome que aparece no cabeçalho, e o que trocá-lo muda. Fica em `<Suspense>`. */
export async function DisplayNameSection() {
  const [user, t] = await Promise.all([
    getCurrentUser(),
    getTranslations("Account.name"),
  ]);
  if (!user) return null;

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <DisplayNameForm name={user.display_name} />
    </section>
  );
}
