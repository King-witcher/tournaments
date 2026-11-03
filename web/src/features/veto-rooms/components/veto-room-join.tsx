"use client";

import { Form, InlineLoading, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useEffect, useRef, useState } from "react";

import { VETO_ID } from "@/features/matches/lib/match-target";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Section } from "@/shared/ui/section";
import { TextInput } from "@/shared/ui/text-input";
import { useVetoRoom } from "../hooks/use-veto-room";

/** O mesmo teto do nome de uma conta e da tabela. */
const MAX_NAME = 32;

/**
 * Entrar no lugar deste link. Quem tem sessão entra sozinho, com o nome da
 * conta; quem não tem (ou a sessão venceu), digita um nome e segue.
 */
export function VetoRoomJoin({ signedIn }: { signedIn: boolean }) {
  const t = useTranslations("VetoRoom.join");
  const errorMessage = useApiErrorMessage();
  const { token, adopt, headers } = useVetoRoom();
  const { ref, canSubmit } = useFormValidity();
  const params = { path: { token } };
  const [name, setName] = useState("");
  const [asksName, setAsksName] = useState(!signedIn);
  const started = useRef(false);

  useEffect(() => {
    if (!signedIn || started.current) return;
    started.current = true;
    callApi(
      api.POST("/veto-rooms/{token}/join", { params, headers: headers() }),
    ).then((outcome) => {
      if (outcome.ok) adopt(outcome.data);
      else setAsksName(true);
    });
  });

  const [error, submit, pending] = useActionState<string | undefined>(
    async () => {
      const outcome = await callApi(
        api.POST("/veto-rooms/{token}/join", {
          params,
          headers: headers(),
          body: { name },
        }),
      );
      if (outcome.ok) {
        adopt(outcome.data);
        // O formulário some: o foco vai para o passo que abre no lugar dele.
        focusIfLost(VETO_ID, { preventScroll: true });
        return undefined;
      }
      return errorMessage(outcome.code, outcome.params);
    },
    undefined,
  );

  if (!asksName) {
    return <InlineLoading description={t("entering")} />;
  }
  return (
    <Section title={t("title")}>
      <Form ref={ref} action={submit}>
        <Stack gap={6}>
          <TextInput
            id="veto-room-name"
            labelText={t("name")}
            name="name"
            required
            maxLength={MAX_NAME}
            autoComplete="off"
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          {error && <FormError>{error}</FormError>}
          <FormActions
            submitLabel={t("submit")}
            pending={pending}
            disabled={!canSubmit}
          />
        </Stack>
      </Form>
    </Section>
  );
}
