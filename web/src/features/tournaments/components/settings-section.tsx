"use client";

import { useTranslations } from "next-intl";
import type { ReactNode, Ref, RefObject } from "react";

import { Button } from "@/shared/ui/button";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import {
  type SettingsSection as SectionKey,
  sectionHeadingId,
} from "../lib/settings-sections";

/** O que o formulário da seção precisa do hook dela. */
export type SectionFormState = {
  formRef: RefObject<HTMLFormElement | null>;
  canSubmit: boolean;
  pending: boolean;
  /** O erro que não é de um campo daqui, com a seção onde se corrige. */
  formError?: { message: string; fixIn?: SectionKey };
  /** Acabou de salvar e nada mudou desde então. */
  done: boolean;
};

type Props = {
  section: SectionKey;
  form: SectionFormState;
  /** Por que a seção (ou parte dela) não muda agora: o `Callout` antes dos campos. */
  lock?: ReactNode;
  /** Nada aqui muda no estado de agora: os campos só leitura e sem o Salvar. */
  readOnly?: boolean;
  /** Salvar (o Enter num campo ou o botão), com o formulário válido. */
  onSave: () => void;
  /** O primário, para devolver o foco a ele (depois de um Modal que ele abriu). */
  submitRef?: Ref<HTMLButtonElement>;
  children: ReactNode;
  /** Depois do formulário, fora dele (a capa, que se salva sozinha). */
  after?: ReactNode;
};

/**
 * Uma seção da aba Configurações do torneio, como as da comunidade: o título,
 * o motivo do que não muda num `Callout` antes dos campos, os campos e o
 * Salvar dela, desabilitado enquanto um campo recusa o que tem. Erro de outra
 * seção (as séries daqui pedem mais mapas que o pool salvo) aparece acima do
 * botão, com "Ir para" a seção onde corrigir; salvo, "Salvo." até a próxima
 * mudança.
 */
export function SettingsSection({
  section,
  form,
  lock,
  readOnly = false,
  onSave,
  submitRef,
  children,
  after,
}: Props) {
  const t = useTranslations("TournamentSettings");
  const fixIn = form.formError?.fixIn;
  // O foco rola até o título, no meio da tela (no topo, ficaria embaixo do
  // cabeçalho fixo).
  const goTo = (target: SectionKey) =>
    document.getElementById(sectionHeadingId(target))?.focus();

  return (
    <Section
      title={t(`sections.${section}`)}
      id={sectionHeadingId(section)}
      focusable
    >
      <Form
        ref={form.formRef}
        onSubmit={(event) => {
          event.preventDefault();
          if (!readOnly && !form.pending && form.canSubmit) onSave();
        }}
      >
        <Stack gap={6}>
          {lock && <Notice kind="info">{lock}</Notice>}
          {children}
          {form.formError && (
            <FormError
              action={
                fixIn &&
                fixIn !== section && (
                  <Button kind="ghost" size="sm" onClick={() => goTo(fixIn)}>
                    {t("goTo", { section: t(`sections.${fixIn}`) })}
                  </Button>
                )
              }
            >
              {form.formError.message}
            </FormError>
          )}
          {form.done && <Notice kind="success">{t("saved")}</Notice>}
          {!readOnly && (
            <FormActions
              submitRef={submitRef}
              pending={form.pending}
              disabled={!form.canSubmit}
            />
          )}
        </Stack>
      </Form>
      {after}
    </Section>
  );
}
