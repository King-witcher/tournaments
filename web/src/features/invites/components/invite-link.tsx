"use client";

import { CodeSnippet } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useEffect, useRef } from "react";

import styles from "./invite-link.module.scss";

/**
 * Link recém-gerado (de convite ou de inscrição), com o botão de copiar do
 * Carbon. Aparece uma vez (a API só guarda o hash), então recebe o foco ao
 * aparecer: o próximo passo de quem gerou é copiar.
 */
export function InviteLink({ link }: { link: string }) {
  const t = useTranslations("Tenant.invites");
  const wrapper = useRef<HTMLDivElement>(null);

  useEffect(() => {
    wrapper.current?.querySelector("button")?.focus();
  }, []);

  return (
    <div ref={wrapper} className={styles.link}>
      <p className={styles.label}>{t("link")}</p>
      <CodeSnippet
        type="single"
        aria-label={t("link")}
        copyButtonDescription={t("copy")}
        feedback={t("copied")}
      >
        {link}
      </CodeSnippet>
      <p className={styles.hint}>{t("createdHint")}</p>
    </div>
  );
}
