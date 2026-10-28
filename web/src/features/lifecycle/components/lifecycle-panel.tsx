import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { type Transition, TransitionButton } from "./transition-button";
import { UndoStartControl } from "./undo-start-control";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
  /** Botão de quem monta o quadro (editar), depois das ações do ciclo. */
  extra?: ReactNode;
};

/**
 * Para organizer e owner, depois de publicar: em que pé está o ciclo e os
 * botões de abrir o check-in, reabrir as inscrições, iniciar e desfazer. Vai
 * dentro do quadro da organização (`TournamentActions`).
 */
export function LifecyclePanel({ tenant, tournament, now, extra }: Props) {
  const t = useTranslations("Lifecycle");
  const { status, slug } = tournament;
  const hasCheckIn = tournament.check_in_minutes != null;
  const opensAt = tournament.check_in_opens_at;
  const button = (
    transition: Transition,
    label: string,
    options: { confirm?: string; primary?: boolean } = {},
  ) => (
    <TransitionButton
      tenant={tenant}
      slug={slug}
      transition={transition}
      label={label}
      confirm={options.confirm}
      variant={options.primary ? "default" : "outline"}
    />
  );

  let hint: ReactNode = null;
  let actions: ReactNode = null;
  if (status === "registration" && hasCheckIn) {
    // Inscrição com a abertura já no passado: o organizador reabriu.
    const suspended = opensAt != null && new Date(opensAt) <= now;
    hint = suspended
      ? t("suspended")
      : opensAt &&
        t.rich("opensAt", { date: () => <LocalDateTime value={opensAt} /> });
    actions = button("open_check_in", t("openCheckIn"), { primary: true });
  } else if (status === "registration") {
    hint = t("noCheckIn");
    actions = button("start", t("start"), {
      confirm: t("startConfirmNoCheckIn"),
      primary: true,
    });
  } else if (status === "check_in") {
    hint = t("checkInOpen");
    actions = (
      <>
        {button("start", t("start"), {
          confirm: t("startConfirm"),
          primary: true,
        })}
        {button("reopen_registration", t("reopenRegistration"), {
          confirm: t("reopenConfirm"),
        })}
      </>
    );
  } else if (status === "in_progress") {
    hint = t("inProgress");
    actions = (
      <UndoStartControl
        tenant={tenant}
        slug={slug}
        message={
          hasCheckIn ? t("undoStartConfirm") : t("undoStartConfirmNoCheckIn")
        }
      />
    );
  } else if (status === "finished") {
    hint = t("finished");
  } else {
    return extra ?? null;
  }

  return (
    <>
      {hint && <p className="text-sm">{hint}</p>}
      <div className="flex flex-wrap items-start gap-3">
        {actions}
        {extra}
      </div>
    </>
  );
}
