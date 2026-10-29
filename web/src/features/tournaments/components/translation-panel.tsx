import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId } from "react";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { Notice } from "@/shared/ui/notice";
import type { FieldErrors } from "../lib/form-errors";
import type { TranslationDraft, TranslationValues } from "../lib/form-values";
import { TournamentTextFields } from "./tournament-text-fields";
import styles from "./translation-panel.module.scss";

type Locale = components["schemas"]["Locale"];

type Props = {
  locale: Locale;
  /** O idioma original, para dizer o que aparece sem tradução. */
  original: Locale;
  draft: TranslationDraft;
  onChange: (patch: Partial<TranslationDraft>) => void;
  errors: FieldErrors<keyof TranslationValues>;
  /** O torneio já tem esta tradução salva: desligar a apaga ao salvar. */
  saved: boolean;
};

/**
 * A aba de um idioma que não é o original. Desligada, diz o que quem lê
 * nesse idioma vê e oferece "Adicionar tradução"; ligada, os campos (nome
 * obrigatório) e "Remover tradução". O texto não se perde ao desligar: ligar
 * de novo antes de salvar o traz de volta. O foco acompanha a troca.
 */
export function TranslationPanel({
  locale,
  original,
  draft,
  onChange,
  errors,
  saved,
}: Props) {
  const t = useTranslations("TournamentForm.translation");
  const tLocales = useTranslations("Locales");
  const nameId = useId();
  const addId = useId();
  const toggle = (enabled: boolean) => {
    onChange({ enabled });
    // O botão usado some: o foco vai para o que aparece no lugar dele.
    requestAnimationFrame(() =>
      document.getElementById(enabled ? nameId : addId)?.focus(),
    );
  };

  if (!draft.enabled) {
    return (
      <Stack gap={5}>
        {saved && <Notice kind="warning">{t("willDelete")}</Notice>}
        <p>
          {t("off", {
            language: tLocales(locale),
            original: tLocales(original),
          })}
        </p>
        <div>
          <Button id={addId} kind="tertiary" onClick={() => toggle(true)}>
            {t("add")}
          </Button>
        </div>
      </Stack>
    );
  }

  return (
    <Stack gap={6}>
      <p className={styles.hint}>{t("hint")}</p>
      <TournamentTextFields
        locale={locale}
        values={draft}
        onChange={onChange}
        errors={errors}
        translation
        nameId={nameId}
      />
      <div>
        <Button kind="danger--ghost" onClick={() => toggle(false)}>
          {t("remove")}
        </Button>
      </div>
    </Stack>
  );
}
