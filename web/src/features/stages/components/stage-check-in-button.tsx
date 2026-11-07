"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button, type ButtonKind } from "@/shared/ui/button";
import { OverflowActions } from "@/shared/ui/overflow-actions";

type Props = {
  tenant: string;
  slug: string;
  stage: number;
  /** O time que faz ou desfaz o check-in. */
  team: number;
  /** O nome do time, na pergunta de desmarcar. */
  name: string;
  checkedIn: boolean;
  /**
   * `self`: o time de quem vê, com "Fazer check-in" e "Desfazer check-in".
   * `row`: o ⋯ da linha de um time na lista (a organização), e desmarcar
   * pergunta antes. `tile`: a tarefa de "Esperando você", só para fazer.
   */
  variant: "self" | "row" | "tile";
  /**
   * Em `self`: `primary` para quem joga, `tertiary` para quem também
   * organiza (o primário do bloco é o passo da organização), como no
   * cabeçalho do torneio.
   */
  kind?: ButtonKind;
  /** Em `tile`: o alvo do foco quando a tarefa some. */
  focusAfter?: string;
};

/**
 * O check-in de um time na fase seguinte, ou desfazê-lo, com as palavras do
 * check-in do torneio. Quem joga pelo time e a organização podem os dois, até
 * a fase começar.
 */
export function StageCheckInButton({
  tenant,
  slug,
  stage,
  team,
  name,
  checkedIn,
  variant,
  kind = "primary",
  focusAfter,
}: Props) {
  const t = useTranslations("Stages.checkIn");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const self = variant === "self";
  const path = { slug: tenant, tournament: slug, stage, team };
  const route =
    "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/teams/{team}/check-in";
  const refresh = () => {
    router.refresh();
    if (focusAfter) focusIfLost(focusAfter);
  };
  const mark = () => run(() => api.POST(route, { params: { path } }), refresh);
  const unmark = () =>
    run(() => api.DELETE(route, { params: { path } }), refresh);

  // Na linha da lista, o ⋯ (como nos inscritos): o botão com texto não cabe
  // ao lado das colunas no celular e ficava fora da tela, na rolagem da tabela.
  if (variant === "row") {
    return (
      <ActionWithNote error={error} align="end">
        <OverflowActions
          size="sm"
          pending={pending}
          label={t("actionsFor", { name })}
          items={[
            checkedIn
              ? {
                  label: t("unmark"),
                  destructive: true,
                  confirm: {
                    question: t("unmarkConfirm", { name, number: stage }),
                  },
                  onSelect: unmark,
                }
              : { label: t("mark"), onSelect: mark },
          ]}
        />
      </ActionWithNote>
    );
  }
  return (
    <ActionWithNote error={error} align={self ? "start" : "end"}>
      {checkedIn ? (
        <Button kind="tertiary" pending={pending} onClick={unmark}>
          {t("undo")}
        </Button>
      ) : (
        <Button
          kind={self ? kind : "tertiary"}
          size={self ? "md" : "sm"}
          pending={pending}
          onClick={mark}
        >
          {t("do")}
        </Button>
      )}
    </ActionWithNote>
  );
}
