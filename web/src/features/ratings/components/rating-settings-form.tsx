"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId, useRef, useState, useTransition } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { NumberInput } from "@/shared/ui/number-input";

type Settings = components["schemas"]["RatingSettings"];
type Field = keyof Settings;
type Values = { [F in Field]: number | null };

/** O campo de cada erro da API (a ordem dos K vai no K mínimo). */
const FIELD_OF: Record<string, Field> = {
  "rating.initial_invalid": "initial",
  "rating.k_min_invalid": "k_min",
  "rating.k_max_invalid": "k_max",
  "rating.k_decay_invalid": "k_decay",
  "rating.k_order_invalid": "k_min",
};

/** Os limites da API, que cada campo cobra enquanto se digita. */
const LIMITS = {
  initial: { min: 100, max: 3000 },
  k_max: { min: 1, max: 200 },
  k_min: { min: 1, max: 100 },
  k_decay: { min: 1, max: 100 },
} satisfies Record<Field, { min: number; max: number }>;

/** O que foi salvo, para dizer "salvo" enquanto os campos não mudam. */
type Result = FormResult<Field> & { savedValues?: Values };

/**
 * Os parâmetros do Elo (rating inicial e o fator K, do inicial ao mínimo,
 * com a queda). O rating é sempre refeito do zero, então salvar sempre
 * recalcula: o botão abre a confirmação que diz isso (`ConfirmModal`, com o
 * foco no "Não", já que não há campo), e o "Salvar" fica pendente até a
 * resposta.
 */
export function RatingSettingsForm({
  slug,
  initial,
}: {
  slug: string;
  initial: Settings;
}) {
  const t = useTranslations("Ratings.settings");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const submitButton = useRef<HTMLButtonElement>(null);
  const { ref, canSubmit } = useFormValidity();
  const [values, setValues] = useState<Values>(initial);
  const [asking, setAsking] = useState(false);
  const [result, setResult] = useState<Result>({});
  const [pending, startTransition] = useTransition();
  useFocusInvalid(prefix, result);

  const save = () =>
    startTransition(async () => {
      const outcome = await callApi(
        api.PUT("/tenants/{slug}/ratings/settings", {
          params: { path: { slug } },
          body: {
            initial: values.initial ?? 0,
            k_min: values.k_min ?? 0,
            k_max: values.k_max ?? 0,
            k_decay: values.k_decay ?? 0,
          },
        }),
      );
      setAsking(false);
      if (!outcome.ok) {
        setResult({
          error: errorMessage(outcome.code, outcome.params),
          field: FIELD_OF[outcome.code],
        });
        return;
      }
      setValues(outcome.data);
      setResult({ saved: true, savedValues: outcome.data });
      router.refresh();
      requestAnimationFrame(() => submitButton.current?.focus());
    });

  const general = formError(result);
  const saved =
    result.saved &&
    !pending &&
    (Object.keys(values) as Field[]).every(
      (key) => result.savedValues?.[key] === values[key],
    );

  const field = (
    name: Field,
    label: string,
    hint?: string,
    max = LIMITS[name].max,
  ) => (
    <NumberInput
      id={fieldId(prefix, name)}
      name={name}
      labelText={label}
      helperText={hint}
      required
      allowEmpty
      hideSteppers
      disableWheel
      step={1}
      min={LIMITS[name].min}
      max={max}
      value={values[name]}
      onChange={(value) => setValues((v) => ({ ...v, [name]: value }))}
      {...invalidProps(result, name)}
    />
  );

  return (
    <>
      <Form
        ref={ref}
        onSubmit={(event) => {
          event.preventDefault();
          setAsking(true);
        }}
      >
        <Stack gap={6}>
          {field("initial", t("initial"))}
          {field("k_max", t("kMax"), t("kMaxHint"))}
          {/* O mínimo vai até o inicial: o campo já cobra a ordem dos dois. */}
          {field("k_min", t("kMin"), t("kMinHint"), kMinCeiling(values))}
          {field("k_decay", t("kDecay"), t("kDecayHint"))}
          {general && <FormError>{general}</FormError>}
          {saved && <Notice kind="success">{t("saved")}</Notice>}
          <FormActions
            submitRef={submitButton}
            pending={pending}
            disabled={!canSubmit}
          />
        </Stack>
      </Form>
      <ConfirmModal
        open={asking}
        title={t("recompute.title")}
        confirmLabel={t("recompute.submit")}
        onConfirm={() => {
          setAsking(false);
          save();
        }}
        onClose={() => {
          setAsking(false);
          // O Modal sai da árvore e o foco cairia no body.
          requestAnimationFrame(() => submitButton.current?.focus());
        }}
      >
        {t("recompute.text")}
      </ConfirmModal>
    </>
  );
}

/** O maior K mínimo que vale: o limite da API ou o K inicial, o menor. */
function kMinCeiling(values: Values) {
  return Math.min(LIMITS.k_min.max, values.k_max ?? LIMITS.k_min.max);
}
