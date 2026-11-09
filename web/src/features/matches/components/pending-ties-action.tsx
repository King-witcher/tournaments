"use client";

import { useTranslations } from "next-intl";
import { useEffect, useRef, useState } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import { Callout } from "@/shared/ui/callout";
import { OPEN_TIES_PARAM, STANDINGS_TABLE_ID, type Tie } from "../lib/ties";
import styles from "./pending-ties-action.module.scss";
import { TiesModal } from "./ties-modal";

type Props = {
  tenant: string;
  slug: string;
  ties: Tie[];
  /** É a última fase: decididos os empates, o torneio termina. */
  lastStage: boolean;
  /** Veio de um "Decidir": entra com o Modal aberto. */
  autoOpen?: boolean;
};

/**
 * O aviso dos empates a decidir, para a organização: a quantidade e "Decidir",
 * que abre o Modal; nada pendente, nada aparece. `Callout` (informação que
 * carrega com a página, sem fechar nem tomar o foco) porque a decisão pode
 * esperar: um `ActionableNotification` prenderia o foco a cada visita.
 */
export function PendingTiesAction({
  tenant,
  slug,
  ties,
  lastStage,
  autoOpen,
}: Props) {
  const t = useTranslations("Bracket.ties");
  const [open, setOpen] = useState(false);
  const opened = useRef(false);

  // Abre depois de montar (o Modal mora no `document.body`, que o servidor não
  // tem) e tira o parâmetro do endereço, para recarregar não reabrir.
  useEffect(() => {
    if (!autoOpen || opened.current) return;
    opened.current = true;
    const url = new URL(window.location.href);
    url.searchParams.delete(OPEN_TIES_PARAM);
    window.history.replaceState(null, "", url);
    if (ties.length > 0) setOpen(true);
  }, [autoOpen, ties.length]);

  const close = () => {
    setOpen(false);
    // O "Decidir" que abriu some quando o último empate é decidido.
    focusIfLost(STANDINGS_TABLE_ID, { preventScroll: true });
  };

  return (
    <>
      {ties.length > 0 && (
        <Callout
          className={styles.callout}
          kind="warning"
          lowContrast
          title={t("count", { count: ties.length })}
          subtitle={t(lastStage ? "description" : "descriptionStage")}
          actionButtonLabel={t("decide")}
          onActionButtonClick={() => setOpen(true)}
        />
      )}
      {open && (
        <TiesModal
          tenant={tenant}
          slug={slug}
          ties={ties}
          lastStage={lastStage}
          onClose={close}
        />
      )}
    </>
  );
}
