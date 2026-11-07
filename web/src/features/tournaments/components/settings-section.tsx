"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, type Ref, type RefObject, useEffect } from "react";

import { Button } from "@/shared/ui/button";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { useSettingsDrafts } from "../hooks/use-settings-drafts";
import {
  type SettingsSection as SectionKey,
  sectionTitleId,
} from "../lib/settings-sections";
import styles from "./settings-section.module.scss";

/** O que o formulário da seção precisa do hook dela. */
export type SectionFormState = {
  formRef: RefObject<HTMLFormElement | null>;
  canSubmit: boolean;
  pending: boolean;
  /** O erro que não é de um campo daqui, com a seção onde se corrige. */
  formError?: { message: string; fixIn?: SectionKey };
  /** Acabou de salvar e nada mudou desde então. */
  done: boolean;
  /** Algum campo está diferente do salvo. */
  dirty: boolean;
};

type Props = {
  section: SectionKey;
  form: SectionFormState;
  /** O que está salvo, numa linha, embaixo do título. */
  summary: ReactNode;
  /** Por que a seção (ou parte dela) não muda agora: o `Callout` antes dos campos. */
  lock?: ReactNode;
  /** Nada aqui muda no estado de agora: os campos só leitura e sem o Salvar. */
  readOnly?: boolean;
  /** Salvar (o Enter num campo ou o botão), com o formulário válido. */
  onSave: () => void;
  /** O primário, para devolver o foco a ele (depois de um Modal que ele abriu). */
  submitRef?: Ref<HTMLButtonElement>;
  children: ReactNode;
  /**
   * Ao lado do formulário, fora dele (a capa, que se salva sozinha): na
   * metade da direita quando a seção é larga, embaixo quando não é.
   */
  aside?: ReactNode;
};

/**
 * Uma seção das Configurações do torneio, a página inteira dela: o título,
 * com o estado ao lado ("Mudanças não salvas" com o aviso do Carbon, ou "Só
 * leitura"), e embaixo o resumo do que está salvo; depois, o motivo do que
 * não muda num `Callout` antes dos campos, os campos e o Salvar dela,
 * desabilitado enquanto um campo recusa o que tem. Ir a outra seção não perde
 * o que foi mexido. Erro de outra seção (as séries daqui pedem mais mapas que
 * o pool salvo) aparece acima do botão, com "Ir para" a página da seção onde
 * corrigir, que abre com o foco no título; salvo, o foco fica no Salvar e
 * "Salvo." aparece até a próxima mudança. Com `aside`, o formulário fica na
 * metade da esquerda e ele na da direita, como as duas colunas da página.
 */
export function SettingsSection({
  section,
  form,
  summary,
  lock,
  readOnly = false,
  onSave,
  submitRef,
  children,
  aside,
}: Props) {
  const t = useTranslations("TournamentSettings");
  const { goTo, takeArrival } = useSettingsDrafts();
  const fixIn = form.formError?.fixIn;
  const titleId = sectionTitleId(section);

  // Chegou pelo "Ir para": o leitor de tela lê o título e o estado.
  useEffect(() => {
    if (takeArrival(section)) document.getElementById(titleId)?.focus();
  }, [takeArrival, section, titleId]);

  const body = (
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
  );

  return (
    <Section
      id={titleId}
      focusable
      title={
        <span className={styles.head}>
          <span>{t(`sections.${section}`)}</span>
          {form.dirty ? (
            <IconIndicator kind="caution-minor" label={t("unsaved")} />
          ) : (
            readOnly && (
              <StatusTag status="neutral" size="sm">
                {t("readOnly")}
              </StatusTag>
            )
          )}
        </span>
      }
      description={summary}
    >
      {aside ? (
        <div className={styles.halves}>
          <div className={styles.columns}>
            {body}
            {aside}
          </div>
        </div>
      ) : (
        body
      )}
    </Section>
  );
}
