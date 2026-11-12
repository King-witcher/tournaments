"use client";

import { useTranslations } from "next-intl";

import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { LeaveGuard } from "@/shared/ui/leave-guard";
import { Notice } from "@/shared/ui/notice";
import type { FormationBoardState } from "../hooks/use-formation-board";
import styles from "./formation-actions.module.scss";

type Props = {
  tenant: string;
  tournament: string;
  board: FormationBoardState;
};

/**
 * A barra de "Salvar times" e "Cancelar" (que volta à formação da página),
 * presa no pé da área visível enquanto a página rola; "Salvar times" só
 * habilita com mudança não salva, e sair da página pergunta. O erro da API fica
 * junto dos botões, como o aviso de salvo.
 */
export function FormationActions({ tenant, tournament, board }: Props) {
  const t = useTranslations("Team.formation");
  return (
    <>
      <LeaveGuard
        unsaved={board.dirty}
        path={`/${tenant}/${tournament}/teams`}
      />
      <Form
        noValidate
        className={styles.bar}
        onSubmit={(event) => {
          event.preventDefault();
          board.save();
        }}
      >
        <Stack gap={5}>
          {board.saved && <Notice kind="success">{t("saved")}</Notice>}
          {board.error && <FormError>{board.error}</FormError>}
          <FormActions
            submitLabel={t("save")}
            pending={board.pending}
            disabled={!board.dirty}
            onCancel={board.dirty ? board.cancel : undefined}
          />
        </Stack>
      </Form>
    </>
  );
}
