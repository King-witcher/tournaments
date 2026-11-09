import { getTranslations } from "next-intl/server";
import { Panel } from "@/shared/ui/panel";

import { Section } from "@/shared/ui/section";
import { LOGIN_SECTION } from "../lib/login-sections";
import { EmailForms } from "./email-forms";
import { EmailProofForm } from "./email-proof-form";

type Props = {
  email?: string;
  hasPassword: boolean;
  providerLinked: boolean;
  /** A conta só entra pelo email, e esta sessão não digitou o código dele. */
  locked: boolean;
};

/**
 * O email da conta, que já é um jeito de entrar (com um código ou link) e é
 * exigido para criar a senha: o campo já vem com o de agora, para trocar,
 * cadastrar ou remover. Com `locked`, antes a prova do email. O bloco recebe o
 * foco quando o botão de remover some (`tabIndex={-1}`).
 */
export async function EmailLoginSection({
  email,
  hasPassword,
  providerLinked,
  locked,
}: Props) {
  const t = await getTranslations("Account.email");

  return (
    <div id={LOGIN_SECTION.email} tabIndex={-1}>
      <Panel>
        <Section
          title={t("title")}
          description={email ? t("signsIn") : t("none")}
        >
          {locked && email ? (
            <EmailProofForm email={email} />
          ) : (
            <EmailForms
              current={email}
              hasPassword={hasPassword}
              providerLinked={providerLinked}
            />
          )}
        </Section>
      </Panel>
    </div>
  );
}
