"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { submitAction } from "@/shared/lib/submit-action";
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

/** O que foi salvo, para dizer "salvo" enquanto os campos não mudam. */
type Result = FormResult<Field> & { savedValues?: Values };

/**
 * Os parâmetros do Elo (rating inicial e o K, do primeiro mapa ao mínimo,
 * com o decaimento). Salvar recalcula todos os ratings, e o texto avisa.
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
  const [values, setValues] = useState<Values>(initial);

  const [result, submit, pending] = useActionState<Result>(async () => {
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
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: FIELD_OF[outcome.code],
      };
    }
    setValues(outcome.data);
    router.refresh();
    return { saved: true, savedValues: outcome.data };
  }, {});
  useFocusInvalid(prefix, result);
  const general = formError(result);
  const saved =
    result.saved &&
    !pending &&
    (Object.keys(values) as Field[]).every(
      (key) => result.savedValues?.[key] === values[key],
    );

  const field = (name: Field, label: string, hint: string) => (
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
      value={values[name]}
      onChange={(value) => setValues((v) => ({ ...v, [name]: value }))}
      {...invalidProps(result, name)}
    />
  );

  return (
    <Form onSubmit={submitAction(submit)}>
      <Stack gap={6}>
        {field("initial", t("initial"), t("initialHint"))}
        {field("k_max", t("kMax"), t("kMaxHint"))}
        {field("k_min", t("kMin"), t("kMinHint"))}
        {field("k_decay", t("kDecay"), t("kDecayHint"))}
        <p>{t("recompute")}</p>
        {general && <FormError>{general}</FormError>}
        {saved && <Notice kind="success">{t("saved")}</Notice>}
        <FormActions pending={pending} />
      </Stack>
    </Form>
  );
}
