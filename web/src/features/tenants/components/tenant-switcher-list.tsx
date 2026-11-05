"use client";

import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";

import { type Href, Link as IntlLink, usePathname } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { InlineLoading } from "@/shared/ui/carbon/inline-loading";
import { SwitcherDivider, SwitcherItem } from "@/shared/ui/carbon/ui-shell";
import { ErrorText } from "@/shared/ui/error-text";
import { HeaderSwitcher } from "@/shared/ui/header-switcher";
import styles from "./tenant-switcher-list.module.scss";

export type MyTenant = components["schemas"]["MyTenant"];

type Props = {
  /** O que chegou na última abertura, se houve. */
  tenants?: MyTenant[];
  /** Estável (o `setState` do painel): a busca sai uma vez por abertura. */
  onLoad: (tenants: MyTenant[]) => void;
};

/**
 * O SwitcherItem repassa `as` ao Link do UI Shell (que o aceita), mas o tipo
 * dele não declara `as` e tipa `href` como string.
 */
function asIntlLink(href: Href) {
  return { as: IntlLink, href } as unknown as { href: string };
}

/**
 * O conteúdo do painel de comunidades, montado a cada abertura: busca a
 * lista de novo e, enquanto ela não chega, mostra a da abertura anterior (ou
 * o carregando, na primeira). A comunidade da página atual leva
 * `aria-current`, sem marca visível (o Carbon não marca item do painel).
 */
export function TenantSwitcherList({ tenants, onLoad }: Props) {
  const t = useTranslations("Nav.tenants");
  const current = usePathname().split("/")[1];
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let live = true;
    api
      .GET("/tenants")
      .then(({ data }) => {
        if (!live) return;
        if (data) onLoad(data);
        else setFailed(true);
      })
      .catch(() => {
        if (live) setFailed(true);
      });
    return () => {
      live = false;
    };
  }, [onLoad]);

  if (!tenants) {
    return (
      <div className={styles.status}>
        {failed ? (
          <ErrorText>{t("failed")}</ErrorText>
        ) : (
          <InlineLoading description={t("loading")} />
        )}
      </div>
    );
  }
  return (
    <>
      {tenants.length === 0 && <p className={styles.status}>{t("empty")}</p>}
      <HeaderSwitcher label={t("label")}>
        {tenants.map((tenant) => (
          <SwitcherItem
            key={tenant.slug}
            {...asIntlLink(`/${tenant.slug}`)}
            aria-current={tenant.slug === current ? "true" : undefined}
            aria-label={tenant.name}
          >
            {tenant.name}
          </SwitcherItem>
        ))}
        {tenants.length > 0 && <SwitcherDivider />}
        <SwitcherItem {...asIntlLink("/new")} aria-label={t("create")}>
          {t("create")}
        </SwitcherItem>
      </HeaderSwitcher>
    </>
  );
}
