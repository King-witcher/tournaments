"use client";

import type { NewTournamentState } from "../hooks/use-new-tournament";
import { TournamentIdentityFields } from "./tournament-identity-fields";
import { TournamentModeFields } from "./tournament-mode-fields";
import {
  type SourceItem,
  TournamentSourceField,
} from "./tournament-source-field";

type Props = {
  tenant: string;
  form: NewTournamentState;
  /** Os torneios da comunidade; `undefined` enquanto chegam. */
  tournaments: SourceItem[] | undefined;
  /** Na página: o `?from=` acompanha "Começar de". */
  syncUrl?: boolean;
};

/**
 * Os campos da criação curta, no Modal e na página: nome, endereço (gerado do
 * nome, com a prévia), "Começar de" (sem torneio na comunidade, não aparece)
 * e modalidade, que vem da origem escolhida e muda depois.
 */
export function NewTournamentFields({
  tenant,
  form,
  tournaments,
  syncUrl = false,
}: Props) {
  const { values, onChange, errors } = form;
  const loading = tournaments === undefined;
  // Enquanto a lista chega, a origem que abriu o formulário já aparece.
  const items = tournaments ?? (form.origin ? [form.origin] : []);

  return (
    <>
      <TournamentIdentityFields
        tenant={tenant}
        values={values}
        onChange={onChange}
        errors={errors}
        isNew
      />
      {(loading || items.length > 0) && (
        <TournamentSourceField
          // A lista chegou: o campo começa de novo, com a escolha nela.
          key={loading ? "loading" : "ready"}
          tenant={tenant}
          tournaments={items}
          loading={loading}
          initial={form.origin?.slug}
          onCopy={form.copyFrom}
          syncUrl={syncUrl}
        />
      )}
      <TournamentModeFields
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={false}
      />
    </>
  );
}
