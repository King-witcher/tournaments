"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { useNewTournament } from "../hooks/use-new-tournament";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";
import { NewTournamentFields } from "./new-tournament-fields";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  /** Os mapas ativos: o pool lembrado e o copiado só levam os que existem. */
  maps: Schemas["Map"][];
  /** Os torneios da comunidade, para "Começar de". */
  tournaments: Schemas["TournamentSummary"][];
  /** O torneio de onde começar ao abrir (`?from=`). */
  source: Schemas["Tournament"] | undefined;
};

/**
 * Criar torneio numa página (link direto, `?from=`, ou quem chega sem
 * sessão): os mesmos campos do Modal da comunidade, "Criar" e "Cancelar".
 * Cria sempre rascunho e leva às Configurações dele.
 */
export function NewTournamentForm({
  tenant,
  maps,
  tournaments,
  source,
}: Props) {
  const t = useTranslations("NewTournament");
  const router = useRouter();
  const form = useNewTournament({
    tenant,
    source,
    maps: () => Promise.resolve(maps),
    onCreated: (slug) => {
      router.push(`/${tenant}/${slug}/settings`);
      focusIfLost(TOURNAMENT_TITLE_ID);
    },
  });

  return (
    <Form
      ref={form.formRef}
      onSubmit={(event) => {
        event.preventDefault();
        if (!form.pending && form.canSubmit) form.submit();
      }}
    >
      <Stack gap={6}>
        <NewTournamentFields
          tenant={tenant}
          form={form}
          tournaments={tournaments}
          syncUrl
        />
        {form.formError && <FormError>{form.formError}</FormError>}
        <FormActions
          submitLabel={t("submit")}
          pending={form.pending}
          disabled={!form.canSubmit}
          cancelHref={`/${tenant}`}
        />
      </Stack>
    </Form>
  );
}
