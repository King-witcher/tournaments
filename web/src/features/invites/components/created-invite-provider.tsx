"use client";

import { Stack } from "@carbon/react";
import { createContext, type ReactNode, useContext, useState } from "react";

import { Notice } from "@/shared/ui/notice";
import { InviteLink } from "./invite-link";

type Outcome = { link: string } | { error: string };

const CreatedInviteContext = createContext<(outcome: Outcome) => void>(
  () => {},
);

/**
 * Para o "Gerar link" da tabela de convites: entrega o link criado (ou o
 * erro, já traduzido) para aparecer acima da tabela.
 */
export function useReportCreatedInvite() {
  return useContext(CreatedInviteContext);
}

/**
 * Mostra, acima da tabela de convites, o link recém-gerado (uma vez: a API só
 * guarda o hash) ou o erro de gerar. O botão fica na barra da tabela (ou no
 * vazio dela) e muda de lugar quando a primeira linha chega; o resultado
 * fica aqui em cima, que não muda.
 */
export function CreatedInviteProvider({ children }: { children: ReactNode }) {
  const [outcome, setOutcome] = useState<Outcome>();

  return (
    <CreatedInviteContext value={setOutcome}>
      <Stack gap={6}>
        {outcome &&
          ("link" in outcome ? (
            // `key`: um link novo é outro bloco, que recebe o foco de novo.
            <InviteLink key={outcome.link} link={outcome.link} />
          ) : (
            <Notice kind="error">{outcome.error}</Notice>
          ))}
        {children}
      </Stack>
    </CreatedInviteContext>
  );
}
