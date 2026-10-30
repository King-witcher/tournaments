"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { Button } from "@/shared/ui/button";
import { ButtonLink } from "@/shared/ui/button-link";
import { Notice } from "@/shared/ui/notice";
import { Page } from "@/shared/ui/page";
import { PageHeader } from "@/shared/ui/page-header";
import styles from "./error-page.module.scss";

type Props = {
  title: string;
  /** O que houve, já traduzido e sem detalhe técnico. */
  children: string;
  /** Tentar de novo (o `retry` do `error.tsx`); sem ele, só o caminho para o início. */
  retry?: () => void;
  /** O `digest` do Next, que casa com o log do servidor: vai pequeno, para o suporte. */
  digest?: string;
};

/**
 * Página de erro do site: o título, o que houve num `Notice` de erro e a
 * saída: "Tentar de novo" (quando há `retry`) e o link para o início.
 * Serve ao 404 (`not-found.tsx`), ao erro de execução (`error.tsx`) e, com um
 * provider próprio, às páginas que substituem o layout (`global-error`, `not-found` da raiz).
 * Nunca mostra a mensagem do erro, só a referência.
 */
export function ErrorPage({ title, children, retry, digest }: Props) {
  const t = useTranslations("Errors");
  const nav = useTranslations("Nav");
  return (
    <Page>
      <PageHeader title={title} />
      <Stack gap={5}>
        <Notice kind="error">{children}</Notice>
        <div className={styles.actions}>
          {retry && <Button onClick={retry}>{t("retry")}</Button>}
          <ButtonLink href="/" kind={retry ? "tertiary" : "primary"}>
            {nav("backHome")}
          </ButtonLink>
        </div>
        {digest && (
          <p className={styles.reference}>{t("reference", { digest })}</p>
        )}
      </Stack>
    </Page>
  );
}
