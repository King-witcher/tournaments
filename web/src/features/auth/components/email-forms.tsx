"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { Notice } from "@/shared/ui/notice";
import { ChangeEmailForm } from "./change-email-form";

type Props = {
  current?: string;
  hasPassword: boolean;
  providerLinked: boolean;
};

/**
 * O formulário do email e o aviso de que o código o confirmou. A `key` remonta o
 * formulário quando o email da conta muda (o refresh mantém o estado do campo);
 * o aviso fica aqui fora para sobreviver a isso.
 */
export function EmailForms({ current, hasPassword, providerLinked }: Props) {
  const t = useTranslations("Account.emailConfirm");
  const [confirmed, setConfirmed] = useState(false);

  return (
    <Stack gap={6}>
      {confirmed && (
        <Notice kind="success" title={t("doneTitle")}>
          {t("done")}
        </Notice>
      )}
      <ChangeEmailForm
        key={current ?? ""}
        current={current}
        hasPassword={hasPassword}
        providerLinked={providerLinked}
        onConfirmed={() => setConfirmed(true)}
      />
    </Stack>
  );
}
