"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { Button } from "@/shared/ui/button";
import { FormModal } from "@/shared/ui/form-modal";
import { TextInput } from "@/shared/ui/text-input";

/** O mesmo limite da API. */
const MAX_NICKNAME = 32;

/** O código que é erro do próprio campo (vai no `invalidText`, não no rodapé). */
const FIELD_ERROR = "member.nickname_invalid";

type Props = {
  slug: string;
  nickname: string;
};

/**
 * "Editar apelido", no cabeçalho da própria página na comunidade: abre o
 * `FormModal` com o apelido. Salvo, o Modal fecha, a página recarrega com o
 * apelido novo e o foco volta ao botão.
 */
export function EditNicknameButton({ slug, nickname }: Props) {
  const t = useTranslations("Member");
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button ref={trigger} kind="tertiary" onClick={() => setOpen(true)}>
        {t("editNickname")}
      </Button>
      {open && (
        <NicknameModal
          slug={slug}
          nickname={nickname}
          trigger={trigger}
          onClose={() => setOpen(false)}
        />
      )}
    </>
  );
}

/** O Modal aberto: um a cada abertura, para começar com o apelido de agora. */
function NicknameModal({
  slug,
  nickname,
  trigger,
  onClose,
}: Props & {
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
}) {
  const t = useTranslations("Member");
  const tField = useTranslations("Tenant.nickname");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const { ref, canSubmit } = useFormValidity();
  const [value, setValue] = useState(nickname);
  /** O último erro foi do campo, e ele não mudou desde então. */
  const [fieldError, setFieldError] = useState(false);

  return (
    <FormModal
      title={t("editNickname")}
      submitLabel={t("nicknameModal.submit")}
      pending={pending}
      submitDisabled={!canSubmit || value.trim() === nickname}
      error={fieldError ? undefined : error}
      returnFocusTo={trigger}
      formRef={ref}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.PATCH("/tenants/{slug}/me", {
              params: { path: { slug } },
              body: { nickname: value },
            }),
          () => {
            onClose();
            router.refresh();
            trigger.current?.focus();
          },
          (code) => setFieldError(code === FIELD_ERROR),
        )
      }
    >
      <TextInput
        labelText={tField("label")}
        required
        maxLength={MAX_NICKNAME}
        autoComplete="nickname"
        value={value}
        onChange={(event) => {
          setValue(event.target.value);
          setFieldError(false);
        }}
        invalid={fieldError}
        invalidText={fieldError ? error : undefined}
      />
    </FormModal>
  );
}
