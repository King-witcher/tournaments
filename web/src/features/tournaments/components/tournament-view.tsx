import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { LoginPrompt } from "@/features/auth/components/login-prompt";
import { loadTournament } from "../server";
import { ErrorBack } from "./error-back";
import { TournamentHeader } from "./tournament-header";
import { TournamentHeaderSwitch } from "./tournament-header-switch";
import { TournamentSectionHeader } from "./tournament-section-header";
import { TournamentTabs } from "./tournament-tabs";

type Props = {
  tenant: string;
  slug: string;
  /** A seção atual (o `children` do `layout.tsx` das seções). */
  children: ReactNode;
};

/**
 * A casca das seções do torneio: o cabeçalho e a seção atual. Na Visão
 * geral, o cabeçalho completo (nome, estado, metadado, capa, ações);
 * nas outras, o compacto, com o nome da seção no título, porque o menu do
 * torneio já diz em que torneio e seção a pessoa está e quem abre a Chave
 * ou a Organização quer o conteúdo dela logo. "Sua situação" e a fase
 * seguinte ficam na Visão geral. Sem sessão, o convite a entrar; com erro, o
 * caminho de volta (e nenhuma seção). Lê a sessão, então fica em
 * `<Suspense>`.
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
  const now = new Date();
  const header = { tenant, me, tournament, now };
  return (
    <>
      <TournamentHeaderSwitch
        overview={<TournamentHeader {...header} />}
        section={<TournamentSectionHeader {...header} />}
      />
      <TournamentTabs
        tenant={tenant}
        tournament={tournament}
        organizer={me.role !== "player"}
      >
        {children}
      </TournamentTabs>
    </>
  );
}
