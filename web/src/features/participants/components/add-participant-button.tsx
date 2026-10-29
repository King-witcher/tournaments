"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormModal } from "@/shared/ui/form-modal";
import { Select } from "@/shared/ui/select";
import { PARTICIPANTS_ID } from "../lib/focus";

type Candidate = { userId: number; nickname: string };

type Props = {
  tenant: string;
  tournament: string;
  /** Membros da comunidade que ainda não estão inscritos. */
  candidates: Candidate[];
  /** Por que não dá para inscrever agora (sem vaga, todos inscritos). */
  blocked?: string;
};

/**
 * "Inscrever alguém", na barra da tabela de inscritos (organizer e owner, em
 * torneio público ou privado, até começar): abre o `FormModal` com a escolha
 * do membro. Sem vaga ou sem membro a inscrever, o botão continua lá e o
 * Modal diz por que não dá e o que fazer (a barra da tabela não tem altura
 * para um motivo embaixo do botão). Ao
 * fechar sem inscrever, o foco volta ao botão; inscrito, a tabela recarrega e
 * o foco vai para ela.
 */
export function AddParticipantButton({
  tenant,
  tournament,
  candidates,
  blocked,
}: Props) {
  const t = useTranslations("Participants");
  const router = useRouter();
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button ref={trigger} onClick={() => setOpen(true)}>
        {t("add.open")}
      </Button>
      {open && (
        <AddParticipantModal
          tenant={tenant}
          tournament={tournament}
          candidates={candidates}
          blocked={blocked}
          trigger={trigger}
          onClose={() => setOpen(false)}
          onDone={() => {
            setOpen(false);
            router.refresh();
            focusIfLost(PARTICIPANTS_ID);
          }}
        />
      )}
    </>
  );
}

/** O Modal aberto: um a cada abertura, para começar sem escolha nem erro. */
function AddParticipantModal({
  tenant,
  tournament,
  candidates,
  blocked,
  trigger,
  onClose,
  onDone,
}: Props & {
  trigger: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onDone: () => void;
}) {
  const t = useTranslations("Participants");
  const { run, pending, error } = useApiAction();
  const [userId, setUserId] = useState("");

  return (
    <FormModal
      title={t("add.title")}
      submitLabel={t("add.submit")}
      pending={pending}
      submitDisabled={blocked !== undefined || !userId}
      error={error}
      returnFocusTo={trigger}
      onClose={onClose}
      onSubmit={() =>
        run(
          () =>
            api.POST("/tenants/{slug}/tournaments/{tournament}/participants", {
              params: { path: { slug: tenant, tournament } },
              body: { user_id: Number(userId) },
            }),
          onDone,
        )
      }
    >
      {blocked !== undefined ? (
        <p>{blocked}</p>
      ) : (
        <>
          <p>{t("add.hint")}</p>
          <Select
            labelText={t("add.member")}
            value={userId}
            onChange={(event) => setUserId(event.target.value)}
          >
            <option value="">{t("add.choose")}</option>
            {candidates.map((c) => (
              <option key={c.userId} value={c.userId}>
                {c.nickname}
              </option>
            ))}
          </Select>
        </>
      )}
    </FormModal>
  );
}
