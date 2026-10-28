import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { MyParticipation } from "@/features/participants/components/my-participation";
import { loadTournament } from "../server";
import { ErrorBack } from "./error-back";
import { TournamentHeader } from "./tournament-header";
import { TournamentTabs } from "./tournament-tabs";

type Props = {
  tenant: string;
  slug: string;
  /** A aba atual (o `children` do `layout.tsx` das abas). */
  children: ReactNode;
};

/**
 * A casca da página do torneio, igual em todas as abas: o cabeçalho (onde
 * estou, estado, o que fazer agora), "Sua situação" de quem joga e as abas,
 * com a atual dentro. Sem sessão, o convite a entrar; com erro, o caminho de
 * volta (e nenhuma aba). Lê a sessão, então fica em `<Suspense>`.
 */
export async function TournamentView({ tenant, slug, children }: Props) {
  const [t, loaded] = await Promise.all([
    getTranslations("Tournament"),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind === "login") {
    return <LoginPrompt message={t("loginRequired")} />;
  }
  if (loaded.kind === "error") {
    return <ErrorBack tenant={tenant} code={loaded.code} />;
  }

  const { me, tournament } = loaded;
  const organizer = me.role !== "player";
  const now = new Date();
  return (
    <>
      <TournamentHeader
        tenant={tenant}
        me={me}
        tournament={tournament}
        now={now}
      />
      <MyParticipation
        tenant={tenant}
        tournament={tournament}
        organizer={organizer}
        now={now}
      />
      <TournamentTabs
        tenant={tenant}
        tournament={tournament}
        organizer={organizer}
      >
        {children}
      </TournamentTabs>
    </>
  );
}
