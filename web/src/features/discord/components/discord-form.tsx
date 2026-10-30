"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { Button } from "@/shared/ui/button";
import { Checkbox } from "@/shared/ui/checkbox";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { FormSection } from "@/shared/ui/form-section";
import { InfoTip } from "@/shared/ui/info-tip";
import { Notice } from "@/shared/ui/notice";
import { Select } from "@/shared/ui/select";
import { TextInput } from "@/shared/ui/text-input";

type Settings = components["schemas"]["DiscordSettings"];
type Locale = components["schemas"]["Locale"];

/** Os códigos que são erro do campo do endereço (vão no `invalidText`). */
const URL_ERRORS = ["discord.url_invalid", "discord.url_required"];

const LOCALES: Locale[] = ["pt", "es"];

/** O endereço que a API aceita (ver `discord/url.rs`), para o campo cobrar enquanto se digita. */
const URL_PATTERN =
  "s*https://(?:ptb.|canary.)?discord(?:app)?.com/api/webhooks/[0-9]{1,20}/[A-Za-z0-9_-]{20,100}s*";

type Props = { slug: string; settings: Settings };

/**
 * O formulário dos avisos no Discord: o endereço do webhook (colado uma vez e
 * nunca mais mostrado: só o id), os avisos ligados, o idioma e as menções,
 * mais "Enviar teste" e "Remover". Com o webhook quebrado, só o endereço
 * novo resolve.
 */
export function DiscordForm({ slug, settings }: Props) {
  const t = useTranslations("Discord");
  const tLocales = useTranslations("Locales");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const { webhook, available } = settings;

  const [url, setUrl] = useState("");
  const [replacing, setReplacing] = useState(false);
  const [events, setEvents] = useState<string[]>(
    webhook?.events ?? available.filter((e) => e.default_on).map((e) => e.code),
  );
  const [locale, setLocale] = useState<Locale>(webhook?.locale ?? "pt");
  const [mention, setMention] = useState(webhook?.mention_players ?? true);
  const asksUrl = !webhook || webhook.broken || replacing;
  const { ref, canSubmit } = useFormValidity();

  const [result, submit, pending] = useActionState<FormResult<"url">>(
    async () => {
      const outcome = await callApi(
        api.PUT("/tenants/{slug}/discord", {
          params: { path: { slug } },
          body: {
            url: asksUrl ? url : null,
            events,
            locale,
            mention_players: mention,
          },
        }),
      );
      if (!outcome.ok) {
        return {
          error: errorMessage(outcome.code, outcome.params),
          field: URL_ERRORS.includes(outcome.code) ? "url" : undefined,
        };
      }
      setUrl("");
      setReplacing(false);
      router.refresh();
      return { saved: true };
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const general = formError(result);

  const urlOk = new RegExp(`^(?:${URL_PATTERN})$`, "u").test(url);
  const test = useApiAction();
  const [tested, setTested] = useState(false);
  const remove = useApiAction();

  const label = (code: string) => {
    const key = `events.${code}` as Parameters<typeof t>[0];
    return t.has(key) ? t(key) : code;
  };
  const toggle = (code: string, on: boolean) =>
    setEvents((now) => (on ? [...now, code] : now.filter((c) => c !== code)));

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={7}>
        {webhook?.broken && (
          <Notice kind="error" title={t("broken.title")}>
            {t("broken.body")}
          </Notice>
        )}
        <Stack gap={5}>
          {webhook && (
            <p>
              {t("current", { id: webhook.webhook_id })}
              {!webhook.broken && (
                <>
                  {" "}
                  <Button
                    kind="ghost"
                    size="sm"
                    onClick={() => setReplacing((now) => !now)}
                  >
                    {replacing ? t("keep") : t("replace")}
                  </Button>
                </>
              )}
            </p>
          )}
          {asksUrl && (
            <TextInput
              id={fieldId(prefix, "url")}
              name="url"
              labelText={
                <InfoTip
                  term={webhook ? t("url.newLabel") : t("url.label")}
                  label={t("url.tipLabel")}
                >
                  {t("url.tip")}
                </InfoTip>
              }
              helperText={t("url.hint")}
              required
              pattern={URL_PATTERN}
              patternMessage={t("url.invalid")}
              autoComplete="off"
              spellCheck={false}
              value={url}
              onChange={(event) => {
                setUrl(event.target.value);
                setTested(false);
              }}
              {...invalidProps(result, "url")}
            />
          )}
          {/* O teste usa o endereço do campo, salvo ou não. */}
          <div>
            <Button
              kind="tertiary"
              pending={test.pending}
              disabled={asksUrl && !urlOk}
              onClick={() => {
                setTested(false);
                test.run(
                  () =>
                    api.POST("/tenants/{slug}/discord/test", {
                      params: { path: { slug } },
                      body: { url: asksUrl ? url.trim() : null, locale },
                    }),
                  () => setTested(true),
                  () => router.refresh(),
                );
              }}
            >
              {t("test")}
            </Button>
          </div>
          {test.error && <ErrorText>{test.error}</ErrorText>}
          {tested && !test.error && (
            <Notice kind="success">{t("tested")}</Notice>
          )}
        </Stack>
        <FormSection legend={t("events.legend")}>
          {available.map((e) => (
            <Checkbox
              key={e.code}
              id={`${prefix}event-${e.code}`}
              labelText={label(e.code)}
              checked={events.includes(e.code)}
              onChange={(on) => toggle(e.code, on)}
            />
          ))}
        </FormSection>
        <Checkbox
          id={`${prefix}mention`}
          labelText={t("mention.label")}
          helperText={t("mention.hint")}
          checked={mention}
          onChange={setMention}
        />
        <Select
          labelText={t("language")}
          value={locale}
          onChange={(event) => setLocale(event.target.value as Locale)}
        >
          {LOCALES.map((l) => (
            <option key={l} value={l}>
              {tLocales(l)}
            </option>
          ))}
        </Select>
        {general && <FormError>{general}</FormError>}
        {result.saved && !pending && (
          <Notice kind="success">{t("saved")}</Notice>
        )}
        <FormActions pending={pending} disabled={!canSubmit} />
        {webhook && (
          <div>
            <ConfirmButton
              size="md"
              pending={remove.pending}
              question={t("removeConfirm")}
              onConfirm={() =>
                remove.run(
                  () =>
                    api.DELETE("/tenants/{slug}/discord", {
                      params: { path: { slug } },
                    }),
                  () => {
                    setReplacing(false);
                    router.refresh();
                    requestAnimationFrame(() =>
                      document.getElementById(fieldId(prefix, "url"))?.focus(),
                    );
                  },
                )
              }
            >
              {t("remove")}
            </ConfirmButton>
          </div>
        )}
        {remove.error && <ErrorText>{remove.error}</ErrorText>}
      </Stack>
    </Form>
  );
}
