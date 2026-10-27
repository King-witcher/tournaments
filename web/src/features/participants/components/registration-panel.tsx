import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { RegistrationButton } from "./registration-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * A inscrição de quem vê, com a inscrição aberta: inscrever-se (só em torneio
 * público, com vaga) ou sair.
 */
export function RegistrationPanel({ tenant, tournament }: Props) {
  const t = useTranslations("Participants.registration");
  if (tournament.status !== "registration") return null;

  const hint = (text: string) => <p className="text-sm opacity-70">{text}</p>;
  if (tournament.registered) {
    return (
      <div className="flex flex-wrap items-center gap-3">
        {hint(t("registered"))}
        <RegistrationButton tenant={tenant} slug={tournament.slug} registered />
      </div>
    );
  }
  if (tournament.visibility === "private") return hint(t("privateHint"));
  if (tournament.participants >= tournament.max_participants) {
    return hint(t("full"));
  }
  return (
    <RegistrationButton
      tenant={tenant}
      slug={tournament.slug}
      registered={false}
    />
  );
}
