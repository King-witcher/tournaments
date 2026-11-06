"use client";

import { UserMultiple } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useCallback, useRef, useState } from "react";

import { HeaderPanelAction } from "@/shared/ui/header-panel-action";
import { CreateTenantModal } from "./create-tenant-modal";
import { type MyTenant, TenantSwitcherList } from "./tenant-switcher-list";

/**
 * Suas comunidades no cabeçalho, para ir de uma a outra sem voltar ao início:
 * ação com o ícone de pessoas, abrindo um painel com a lista (a atual
 * marcada) e "Criar comunidade", que abre o Modal. A lista só é buscada ao
 * abrir (`GET /tenants`): no layout, ela custaria uma leitura da sessão no
 * Redis a cada página. A última que chegou fica aqui, para a abertura
 * seguinte não começar vazia. O Modal fica aqui, fora do painel, que fecha
 * quando o foco vai para ele; ao fechar, o foco volta ao botão da barra.
 */
export function TenantSwitcherPanel() {
  const t = useTranslations("Nav.tenants");
  const trigger = useRef<HTMLButtonElement>(null);
  const [tenants, setTenants] = useState<MyTenant[]>();
  const [creating, setCreating] = useState(false);
  const stopCreating = useCallback(() => setCreating(false), []);

  return (
    <>
      <HeaderPanelAction
        label={t("label")}
        icon={<UserMultiple size={20} />}
        navigation
        triggerRef={trigger}
      >
        <TenantSwitcherList
          tenants={tenants}
          onLoad={setTenants}
          onCreate={() => setCreating(true)}
        />
      </HeaderPanelAction>
      {creating && (
        <CreateTenantModal
          title={t("create")}
          trigger={trigger}
          onClose={stopCreating}
        />
      )}
    </>
  );
}
