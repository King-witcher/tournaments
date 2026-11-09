"use client";

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
import { Form } from "@/shared/ui/carbon/form";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { PageFormLayout } from "@/shared/ui/page-form-layout";
import { TextArea } from "@/shared/ui/text-area";
import { TextInput } from "@/shared/ui/text-input";

type Field = "name" | "slug" | "description";

/** O que a API gravou, para dizer "salvo" enquanto os campos não mudam. */
type Result = FormResult<Field> & {
  savedName?: string;
  savedDescription?: string;
};

/** Os erros que são de um campo: vão no `invalidText` dele, não acima dos botões. */
const FIELD_OF_ERROR: Partial<Record<string, Field>> = {
  "tenant.name_invalid": "name",
  "tenant.description_invalid": "description",
  "tenant.slug_taken": "slug",
  "tenant.slug_invalid": "slug",
  "tenant.slug_reserved": "slug",
};

type Props = {
  slug: string;
  name: string;
  /** Vazia sem descrição. */
  description: string;
};

/**
 * Nome, endereço e descrição da comunidade, com um Salvar, como a seção Geral
 * do torneio. Trocar o endereço quebra links e libera o antigo para outra
 * comunidade, então salvar com ele trocado pede a confirmação que diz isso
 * (`ConfirmModal`, com o foco no "Não"). Salvo, a página vai para o endereço
 * novo; as outras abas vão sozinhas (`moved`, no `LiveConnection`), e esta não
 * precisa do aviso (`originHeaders`).
 */
export function TenantGeneralForm({ slug, name, description }: Props) {
  const t = useTranslations("Tenant.details");
  const tAddress = useTranslations("Tenant.address");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const hub = useRealtimeHub();
  const isClient = useIsClient();
  const prefix = useId();
  const submitButton = useRef<HTMLButtonElement>(null);
  const { ref, canSubmit } = useFormValidity();
  const [nameValue, setNameValue] = useState(name);
  const [slugValue, setSlugValue] = useState(slug);
  const [descriptionValue, setDescriptionValue] = useState(description);
  const [asking, setAsking] = useState(false);
  const [result, setResult] = useState<Result>({});
  const [pending, startTransition] = useTransition();
  useFocusInvalid(prefix, result);

  const save = () =>
    startTransition(async () => {
      const outcome = await callApi(
        api.PATCH("/tenants/{slug}", {
          params: { path: { slug } },
          body: {
            name: nameValue,
            slug: slugValue,
            description: descriptionValue,
          },
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
      if (outcome.data.slug !== slug) {
        router.replace(`/${outcome.data.slug}/settings`);
        router.refresh();
        return;
      }
      const savedName = outcome.data.name;
      const savedDescription = outcome.data.description ?? "";
      setNameValue(savedName);
      setDescriptionValue(savedDescription);
      setResult({ saved: true, savedName, savedDescription });
      router.refresh();
    });

  const general = formError(result);
  const saved =
    result.saved &&
    !pending &&
    result.savedName === nameValue &&
    result.savedDescription === descriptionValue &&
    slugValue === slug;
  // O endereço completo só no navegador (a origem); no servidor, o caminho.
  const address = `${isClient ? window.location.origin : ""}/${slugValue || tAddress("placeholder")}`;

  return (
    <>
      <Form
        ref={ref}
        onSubmit={(event) => {
          event.preventDefault();
          setResult({});
          if (slugValue !== slug) setAsking(true);
          else save();
        }}
      >
        <PageFormLayout
          footer={
            <>
              {general && <FormError>{general}</FormError>}
              {saved && <Notice kind="success">{t("saved")}</Notice>}
              <FormActions
                submitRef={submitButton}
                pending={pending}
                disabled={!canSubmit}
              />
            </>
          }
        >
          <TextInput
            id={fieldId(prefix, "name")}
            labelText={t("name")}
            name="name"
            required
            maxLength={64}
            autoComplete="off"
            value={nameValue}
            onChange={(event) => setNameValue(event.target.value)}
            {...invalidProps(result, "name")}
          />
          <TextInput
            id={fieldId(prefix, "slug")}
            labelText={tAddress("title")}
            helperText={tAddress("hint", { address })}
            name="slug"
            required
            minLength={3}
            maxLength={32}
            pattern="[a-z0-9]+(-[a-z0-9]+)*"
            patternMessage={tAddress("invalid")}
            filter={slugChars}
            autoCapitalize="none"
            autoComplete="off"
            spellCheck={false}
            value={slugValue}
            onChange={(event) => setSlugValue(event.target.value)}
            {...invalidProps(result, "slug")}
          />
          <TextArea
            id={fieldId(prefix, "description")}
            labelText={t("description")}
            helperText={t("descriptionHint")}
            name="description"
            maxLength={2000}
            value={descriptionValue}
            onChange={(event) => setDescriptionValue(event.target.value)}
            {...invalidProps(result, "description")}
          />
        </PageFormLayout>
      </Form>
      <ConfirmModal
        open={asking}
        title={tAddress("submit")}
        confirmLabel={tAddress("submit")}
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
        {tAddress("confirm", { old: slug, address: slugValue })}
      </ConfirmModal>
    </>
  );
}
