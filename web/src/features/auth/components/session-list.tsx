"use client";

import { useTranslations } from "next-intl";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Stack } from "@/shared/ui/carbon/stack";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorText } from "@/shared/ui/error-text";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { StructuredList } from "@/shared/ui/structured-list";
import { parseUserAgent } from "../lib/user-agent";

type Session = components["schemas"]["SessionInfo"];

/** O título da seção, para onde o foco vai quando o botão que agiu some. */
const SECTION_ID = "account-sessions";

/** A chave de "Encerrar as outras" no grupo de botões (as linhas usam o id). */
const ALL_OTHERS = "others";

/**
 * As sessões abertas da conta: a desta requisição primeiro e marcada (sai
 * pelo "Sair" do menu, não por aqui), as outras com "Encerrar" e, ao lado do
 * título, "Encerrar as outras". Ambos pedem confirmação; depois a lista
 * recarrega e o foco vai para a seção.
 */
export function SessionList({ sessions }: { sessions: Session[] }) {
  const t = useTranslations("Account.sessions");
  const router = useRouter();
  const { runAs, buttonFor, error } = useApiAction();
  const others = sessions.filter((s) => !s.current);

  const done = () => {
    router.refresh();
    focusIfLost(SECTION_ID);
  };
  const endOne = (id: string) =>
    runAs(
      id,
      () => api.DELETE("/auth/sessions/{id}", { params: { path: { id } } }),
      done,
      // Já encerrada em outro lugar: a lista está velha.
      (code) => {
        if (code === "auth.session_not_found") router.refresh();
      },
    );
  const endOthers = () =>
    runAs(ALL_OTHERS, () => api.DELETE("/auth/sessions"), done);

  const device = (session: Session) => {
    const { browser, os } = parseUserAgent(session.user_agent);
    const name =
      browser && os
        ? t("device", { browser, os })
        : (browser ?? os ?? t("unknownDevice"));
    return (
      <Stack gap={2}>
        <span>{name}</span>
        {/* Fora do div, o Stack esticaria a etiqueta até a largura máxima dela. */}
        {session.current && (
          <div>
            <StatusTag status="confirmed">{t("current")}</StatusTag>
          </div>
        )}
      </Stack>
    );
  };

  return (
    <Panel>
      <Section
        id={SECTION_ID}
        title={t("title")}
        description={t("description")}
        action={
          others.length > 0 && (
            <ConfirmButton
              size="md"
              {...buttonFor(ALL_OTHERS)}
              question={t("endOthersConfirm")}
              onConfirm={endOthers}
            >
              {t("endOthers")}
            </ConfirmButton>
          )
        }
      >
        <StructuredList
          label={t("title")}
          headers={[t("deviceHeader"), t("sinceHeader"), ""]}
          rows={sessions.map((session) => ({
            key: session.id,
            cells: [
              device(session),
              <LocalDateTime key="since" value={session.created_at} />,
              session.current ? null : (
                <ConfirmButton
                  key="end"
                  {...buttonFor(session.id)}
                  question={t("endConfirm")}
                  onConfirm={() => endOne(session.id)}
                >
                  {t("end")}
                </ConfirmButton>
              ),
            ],
          }))}
        />
        {others.length === 0 && <EmptyState>{t("onlyThis")}</EmptyState>}
        {error && <ErrorText>{error}</ErrorText>}
      </Section>
    </Panel>
  );
}
