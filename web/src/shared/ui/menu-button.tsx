"use client";

import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import { MenuButton as CarbonMenuButton } from "./carbon/menu-button";
import styles from "./overflow-actions.module.scss";
import { Spinner } from "./spinner";
import { type MenuEntry, useMenuActions } from "./use-menu-actions";

type Props = Omit<ComponentProps<typeof CarbonMenuButton>, "children"> & {
  /** As ações do menu (`MenuAction`, opção, escolha, grupo ou `"divider"`). */
  items: MenuEntry[];
  /** Uma das ações está em andamento: o indicador aparece ao lado e os itens travam. */
  pending?: boolean;
};

/**
 * Botão com texto que abre um menu (`MenuButton` do Carbon): várias ações sob
 * um nome comum ("Exportar", "Adicionar"), sem uma principal. Com uma ação
 * principal e outras ao lado, `ComboButton`; ações de um objeto (linha,
 * cabeçalho) em pouco espaço, `OverflowActions`. `kind` `primary`, `tertiary`
 * ou `ghost`; `menuAlignment` posiciona o menu. Os itens seguem as regras do
 * `OverflowActions`: `href` pelo roteador, `confirm` pelo `ConfirmModal` (e o
 * foco volta ao botão), `destructive` em vermelho.
 */
export function MenuButton({ items, pending, size = "md", ...rest }: Props) {
  const t = useTranslations("Common");
  const { renderItems, confirmModal } = useMenuActions(pending);

  return (
    <span className={styles.wrapper} aria-busy={pending || undefined}>
      <CarbonMenuButton size={size} {...rest}>
        {renderItems(items)}
      </CarbonMenuButton>
      {pending && <Spinner label={t("loading")} />}
      {confirmModal}
    </span>
  );
}
