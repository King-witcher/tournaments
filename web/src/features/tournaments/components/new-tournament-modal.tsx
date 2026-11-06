"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, type RefObject, useEffect, useState } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { FormModal } from "@/shared/ui/form-modal";
import { useCreationContext } from "../hooks/use-creation-context";
import { useNewTournament } from "../hooks/use-new-tournament";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";
import { NewTournamentFields } from "./new-tournament-fields";

type Props = {
  tenant: string;
  /** O título: o nome do botão que abriu ("Novo torneio", "Copiar torneio"). */
  title: ReactNode;
  /** O torneio em "Começar de" ao abrir ("Copiar torneio"). */
  source?: components["schemas"]["Tournament"];
  /** O botão que abriu, para o foco voltar a ele. */
  trigger?: RefObject<HTMLElement | null>;
  /** Fechou: sem criar (Cancelar, Esc, X) ou já na página do criado. */
  onClose: () => void;
};

/**
 * Criar torneio num Modal: nome, endereço, "Começar de" e modalidade, e
 * "Criar", que faz um rascunho e leva às Configurações dele. Os torneios da
 * comunidade e os mapas chegam ao abrir. Um a cada abertura, para começar
 * sem valor nem erro; fechado sem criar, o foco volta ao botão que abriu.
 */
export function NewTournamentModal({
  tenant,
  title,
  source,
  trigger,
  onClose,
}: Props) {
  const t = useTranslations("NewTournament");
  const router = useRouter();
  const pathname = usePathname();
  const [openedAt] = useState(pathname);
  const context = useCreationContext(tenant);
  const form = useNewTournament({
    tenant,
    source,
    maps: context.maps,
    onCreated: (slug) => {
      router.push(`/${tenant}/${slug}/settings`);
      focusIfLost(TOURNAMENT_TITLE_ID);
    },
  });

  // Na página do torneio criado, o Modal sai (a página de onde ele abriu
  // pode continuar montada, como a de outro torneio, no "Copiar torneio").
  useEffect(() => {
    if (pathname !== openedAt) onClose();
  }, [pathname, openedAt, onClose]);

  return (
    <FormModal
      title={title}
      submitLabel={t("submit")}
      pending={form.pending}
      submitDisabled={!form.canSubmit}
      error={form.formError}
      returnFocusTo={trigger}
      onClose={onClose}
      onSubmit={form.submit}
      formRef={form.formRef}
    >
      <NewTournamentFields
        tenant={tenant}
        form={form}
        tournaments={context.tournaments}
      />
    </FormModal>
  );
}
