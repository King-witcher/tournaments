"use client";

import {
  ComposedModal,
  ModalBody,
  ModalFooter,
  ModalHeader,
  Stack,
} from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormError } from "@/shared/ui/form-error";
import { Select } from "@/shared/ui/select";
import { PARTICIPANTS_ID } from "../lib/focus";

type Candidate = { userId: number; nickname: string };

type Props = {
  tenant: string;
  tournament: string;
  /** Membros da comunidade que ainda não estão inscritos. */
  candidates: Candidate[];
};

/**
 * "Inscrever alguém", na barra da tabela de inscritos (organizer e owner, em
 * torneio público ou privado): abre o Modal com a escolha do membro. Ao
 * fechar sem inscrever, o foco volta ao botão; inscrito, a tabela recarrega e
 * o foco vai para ela.
 */
export function AddParticipantButton({
  tenant,
  tournament,
  candidates,
}: Props) {
  const t = useTranslations("Participants");
  const router = useRouter();
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  const close = () => {
    setOpen(false);
    // O Modal sai da árvore e o foco cairia no body: volta para o botão.
    requestAnimationFrame(() => trigger.current?.focus());
  };

  return (
    <>
      <Button ref={trigger} onClick={() => setOpen(true)}>
        {t("add.open")}
      </Button>
      {open &&
        createPortal(
          <AddParticipantModal
            tenant={tenant}
            tournament={tournament}
            candidates={candidates}
            onClose={close}
            onDone={() => {
              setOpen(false);
              router.refresh();
              focusIfLost(PARTICIPANTS_ID);
            }}
          />,
          document.body,
        )}
    </>
  );
}

/** O Modal aberto: um a cada abertura, para começar sem escolha nem erro. */
function AddParticipantModal({
  tenant,
  tournament,
  candidates,
  onClose,
  onDone,
}: Props & { onClose: () => void; onDone: () => void }) {
  const t = useTranslations("Participants");
  const tCommon = useTranslations("Common");
  const { run, pending, error } = useApiAction();
  const [userId, setUserId] = useState("");

  const submit = () => {
    if (!userId) return;
    run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/participants", {
          params: { path: { slug: tenant, tournament } },
          body: { user_id: Number(userId) },
        }),
      onDone,
    );
  };

  return (
    <ComposedModal open size="sm" onClose={onClose}>
      <ModalHeader title={t("add.title")} iconDescription={tCommon("close")} />
      <ModalBody>
        <Stack gap={5}>
          <p>{t("add.hint")}</p>
          <Select
            labelText={t("add.member")}
            value={userId}
            onChange={(event) => setUserId(event.target.value)}
            data-modal-primary-focus
          >
            <option value="">{t("add.choose")}</option>
            {candidates.map((c) => (
              <option key={c.userId} value={c.userId}>
                {c.nickname}
              </option>
            ))}
          </Select>
          {error && <FormError>{error}</FormError>}
        </Stack>
      </ModalBody>
      <ModalFooter>
        <Button kind="secondary" onClick={onClose}>
          {tCommon("cancel")}
        </Button>
        <Button pending={pending} disabled={!userId} onClick={submit}>
          {t("add.submit")}
        </Button>
      </ModalFooter>
    </ComposedModal>
  );
}
