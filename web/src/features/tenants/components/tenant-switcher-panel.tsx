"use client";

import { UserMultiple } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { HeaderPanelAction } from "@/shared/ui/header-panel-action";
import { type MyTenant, TenantSwitcherList } from "./tenant-switcher-list";

/**
 * Suas comunidades no cabeçalho, para ir de uma a outra sem voltar ao início:
 * ação com o ícone de pessoas, abrindo um painel com a lista (a atual
 * marcada) e "Criar comunidade". A lista só é buscada ao abrir
 * (`GET /tenants`): no layout, ela custaria uma leitura da sessão no Redis a
 * cada página. A última que chegou fica aqui, para a abertura seguinte não
 * começar vazia.
 */
export function TenantSwitcherPanel() {
  const t = useTranslations("Nav.tenants");
  const [tenants, setTenants] = useState<MyTenant[]>();
  return (
    <HeaderPanelAction label={t("label")} icon={<UserMultiple size={20} />}>
      <TenantSwitcherList tenants={tenants} onLoad={setTenants} />
    </HeaderPanelAction>
  );
}
