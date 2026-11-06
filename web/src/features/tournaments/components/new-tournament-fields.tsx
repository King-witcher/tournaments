"use client";

import { useEffect, useRef } from "react";

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
 * Os campos da criação curta, no Modal e na página: "Começar de" primeiro
 * (sem torneio na comunidade, não aparece), porque é por onde se decide e,
 * depois do nome vazio, o obrigatório já ficaria vermelho; nome, endereço
 * (gerado do nome, com a prévia) e modalidade, que vem da origem escolhida e
 * muda depois.
 */
export function NewTournamentFields({
  tenant,
  form,
  tournaments,
  syncUrl = false,
}: Props) {
  const { values, onChange, errors, formRef } = form;
  const loading = tournaments === undefined;
  // Enquanto a lista chega, a origem que abriu o formulário já aparece.
  const items = tournaments ?? (form.origin ? [form.origin] : []);
  const withSource = loading || items.length > 0;
  // A lista chegou vazia: o campo, que o Modal abriu com o foco, sai, e o
  // foco perdido vai para o nome.
  const hadSource = useRef(withSource);
  useEffect(() => {
    const removed = hadSource.current && !withSource;
    hadSource.current = withSource;
    const active = document.activeElement;
    if (removed && (active === null || active === document.body)) {
      formRef.current?.querySelector<HTMLElement>('[name="name"]')?.focus();
    }
  }, [withSource, formRef]);

  return (
    <>
      {withSource && (
        <TournamentSourceField
          tenant={tenant}
          tournaments={items}
          loading={loading}
          initial={form.origin?.slug}
          onCopy={form.copyFrom}
          syncUrl={syncUrl}
        />
      )}
      <TournamentIdentityFields
        tenant={tenant}
        values={values}
        onChange={onChange}
        errors={errors}
        isNew
      />
      <TournamentModeFields
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={false}
      />
    </>
  );
}
