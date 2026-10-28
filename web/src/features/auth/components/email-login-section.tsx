import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { ChangeEmailForm } from "./change-email-form";

type Props = { email?: string; hasPassword: boolean };

/**
 * O email da conta, que recupera a senha e é exigido para criá-la: o campo
 * já vem com o de agora, para trocar, cadastrar ou remover. A `key` remonta o
 * formulário quando o email da conta muda (o refresh mantém o estado do
 * campo). O bloco recebe o foco quando o botão de remover some
 * (`tabIndex={-1}`).
 */
export async function EmailLoginSection({ email, hasPassword }: Props) {
  const t = await getTranslations("Account.email");

  return (
    <div id={LOGIN_SECTION.email} tabIndex={-1}>
      <Section
        title={t("title")}
        description={
          !email ? t("none") : hasPassword ? t("recovers") : t("optional")
        }
      >
        <ChangeEmailForm
          key={email ?? ""}
          current={email}
          hasPassword={hasPassword}
        />
      </Section>
    </div>
  );
}
