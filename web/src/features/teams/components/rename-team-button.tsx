"use client";

import { Edit } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { IconButton } from "@/shared/ui/icon-button";
import type { Formation } from "../lib/formation";
import { RenameTeamModal } from "./rename-team-modal";

type Props = {
  tenant: string;
  tournament: string;
  team: components["schemas"]["Team"];
  /**
   * Na edição da formação, recebe a formação nova; sem ele (as listas do
   * servidor), a página recarrega.
   */
  onDone?: (formation: Formation) => void;
};

/**
 * "Renomear", só com o ícone, no cabeçalho do time: para o capitão no próprio
 * time e para a organização na formação. Abre o `RenameTeamModal`; depois de
 * gravar, o foco volta para este botão, que continua lá.
 */
export function RenameTeamButton({ tenant, tournament, team, onDone }: Props) {
  const t = useTranslations("Team.rename");
  const router = useRouter();
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <IconButton
        ref={trigger}
        size="sm"
        label={t("open")}
        onClick={() => setOpen(true)}
      >
        <Edit />
      </IconButton>
      {open && (
        <RenameTeamModal
          tenant={tenant}
          tournament={tournament}
          team={team}
          returnFocusTo={trigger}
          onClose={() => setOpen(false)}
          onDone={(formation) => {
            setOpen(false);
            if (onDone) onDone(formation);
            else router.refresh();
            requestAnimationFrame(() => trigger.current?.focus());
          }}
        />
      )}
    </>
  );
}
