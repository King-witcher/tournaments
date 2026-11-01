"use client";

import { ComboButton as CarbonComboButton } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import styles from "./overflow-actions.module.scss";
import { Spinner } from "./spinner";
import { type MenuEntry, useMenuActions } from "./use-menu-actions";

type Props = Omit<
  ComponentProps<typeof CarbonComboButton>,
  "children" | "translateWithId"
> & {
  /** As ações secundárias, no menu da seta (as mesmas do `MenuButton`). */
  items: MenuEntry[];
  /** A ação principal está em andamento: ignora o clique, mostra o indicador e trava o menu. */
  pending?: boolean;
};

/**
 * A ação principal com as alternativas numa seta ao lado (`ComboButton` do
 * Carbon): "Salvar" e, no menu, "Salvar como rascunho". Sempre `primary`.
 * Várias ações sem uma principal, `MenuButton`. O tooltip da seta é "Mais
 * ações", traduzido. `pending` age como o do `Button` (ignora o clique, sem
 * `disabled`, para o foco ficar), mas o indicador fica ao lado: o Carbon não
 * deixa pôr nada dentro do botão principal. Os itens seguem as regras do
 * `MenuButton`.
 */
export function ComboButton({
  items,
  pending,
  onClick,
  size = "md",
  ...rest
}: Props) {
  const t = useTranslations("Common");
  const { renderItems, confirmModal } = useMenuActions(pending);

  return (
    <span className={styles.wrapper} aria-busy={pending || undefined}>
      <CarbonComboButton
        size={size}
        // Só existe o id "carbon.combo-button.additional-actions".
        translateWithId={() => t("moreActions")}
        onClick={(event) => {
          if (!pending) onClick?.(event);
        }}
        {...rest}
      >
        {renderItems(items)}
      </CarbonComboButton>
      {pending && <Spinner label={t("loading")} />}
      {confirmModal}
    </span>
  );
}
