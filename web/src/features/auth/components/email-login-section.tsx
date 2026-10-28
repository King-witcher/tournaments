import { getTranslations } from "next-intl/server";

import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { ChangeEmailForm } from "./change-email-form";

type Props = { email?: string; hasPassword: boolean };

/**
 * O email da conta, que recupera a senha e é exigido para desvincular o
 * Discord: o de agora e o formulário para cadastrar ou trocar. Conta com
 * senha e sem email (as de antes do email obrigatório) vê o aviso do que
 * perde sem ele, antes dos campos.
 */
export async function EmailLoginSection({ email, hasPassword }: Props) {
  const t = await getTranslations("Account.email");
  const missing = !email && hasPassword;

  return (
    <Section
      title={t("title")}
      description={
        email ? t("current", { email }) : missing ? undefined : t("none")
      }
    >
      {missing && (
        <Notice kind="warning" title={t("missingTitle")}>
          {t("missing")}
        </Notice>
      )}
      <ChangeEmailForm current={email} hasPassword={hasPassword} />
    </Section>
  );
}
