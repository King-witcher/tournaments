"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId, useRef, useState, useTransition } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { useIsClient } from "@/shared/hooks/use-is-client";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { slugChars } from "@/shared/lib/input-filters";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { TextInput } from "@/shared/ui/text-input";

/** Os erros que são do campo: vão no `invalidText` dele, não acima do botão. */
const FIELD_OF_ERROR: Partial<Record<string, "slug">> = {
  "tenant.slug_taken": "slug",
  "tenant.slug_invalid": "slug",
  "tenant.slug_reserved": "slug",
};

/**
 * O endereço da comunidade. Trocar quebra links e libera o antigo para outra
 * comunidade, então o botão abre a confirmação que diz isso (`ConfirmModal`,
 * com o foco no "Não"). Salvo, a página vai para o endereço novo; as outras
 * abas vão sozinhas (`moved`, no `LiveConnection`), e esta não precisa do
 * aviso (`originHeaders`).
 */
export function TenantAddressForm({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.address");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const hub = useRealtimeHub();
  const isClient = useIsClient();
  const prefix = useId();
  const submitButton = useRef<HTMLButtonElement>(null);
  const { ref, canSubmit } = useFormValidity();
  const [value, setValue] = useState(slug);
  const [asking, setAsking] = useState(false);
  const [result, setResult] = useState<FormResult<"slug">>({});
  const [pending, startTransition] = useTransition();
  useFocusInvalid(prefix, result);

  const save = () =>
    startTransition(async () => {
      const outcome = await callApi(
        api.PATCH("/tenants/{slug}", {
          params: { path: { slug } },
          body: { slug: value },
          headers: hub?.originHeaders(),
        }),
      );
      if (!outcome.ok) {
        setResult({
          error: errorMessage(outcome.code, outcome.params),
          field: FIELD_OF_ERROR[outcome.code],
        });
        return;
      }
      router.replace(`/${outcome.data.slug}/settings`);
      router.refresh();
    });

  const general = formError(result);
  // O endereço completo só no navegador (a origem); no servidor, o caminho.
  const address = `${isClient ? window.location.origin : ""}/${value || t("placeholder")}`;

  return (
    <>
      <Form
        ref={ref}
        onSubmit={(event) => {
          event.preventDefault();
          setResult({});
          setAsking(true);
        }}
      >
        <Stack gap={6}>
          <TextInput
            id={fieldId(prefix, "slug")}
            labelText={t("title")}
            hideLabel
            helperText={t("hint", { address })}
            name="slug"
            required
            minLength={3}
            maxLength={32}
            pattern="[a-z0-9]+(-[a-z0-9]+)*"
            patternMessage={t("invalid")}
            filter={slugChars}
            autoCapitalize="none"
            autoComplete="off"
            spellCheck={false}
            value={value}
            onChange={(event) => setValue(event.target.value)}
            {...invalidProps(result, "slug")}
          />
          {general && <FormError>{general}</FormError>}
          <FormActions
            submitLabel={t("submit")}
            submitRef={submitButton}
            pending={pending}
            disabled={!canSubmit || value === slug}
          />
        </Stack>
      </Form>
      <ConfirmModal
        open={asking}
        title={t("submit")}
        confirmLabel={t("submit")}
        destructive={false}
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
        {t("confirm", { old: slug, address: value })}
      </ConfirmModal>
    </>
  );
}
