import type { ReactNode } from "react";

import { apiErrorMessage } from "@/shared/api/errors";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Notice } from "@/shared/ui/notice";
import { loadTournament } from "../server";
import { TournamentSettingsProvider } from "./tournament-settings-provider";

type Props = {
  tenant: string;
  slug: string;
  /** A página da seção aberta (`TournamentSettingsSection`). */
  children: ReactNode;
};

/**
 * As Configurações do torneio, para organizer e owner, no `layout.tsx` delas:
 * o torneio salvo e o que não foi salvo de cada seção
 * (`TournamentSettingsProvider`), em volta da página da seção aberta, na
 * largura toda da página (como a chave e as tabelas). O layout não remonta ao
 * trocar de seção pelo menu, e o que foi mexido continua lá. Para quem joga,
 * o aviso de que a seção não é dele, sem a página. Fica em `<Suspense>`.
 */
export async function TournamentSettings({ tenant, slug, children }: Props) {
  const loaded = await loadTournament(tenant, slug);
  if (loaded.kind !== "ok") return null;
  if (loaded.me.role === "player") {
    return (
      <Notice kind="error">{await apiErrorMessage("tenant.forbidden")}</Notice>
    );
  }

  return (
    <Grid>
      <Column sm={4} md={8} lg={16}>
        <TournamentSettingsProvider
          tenant={tenant}
          tournament={loaded.tournament}
        >
          {children}
        </TournamentSettingsProvider>
      </Column>
    </Grid>
  );
}
