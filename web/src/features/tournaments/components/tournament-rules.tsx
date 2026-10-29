import { Stack } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { UserText } from "@/shared/ui/user-text";
import { rulesSource } from "../lib/localize";
import { textsEditable } from "../lib/status";
import { loadTournament } from "../server";
import { RulesEditor } from "./rules-editor";
import { TournamentAutoRules } from "./tournament-auto-rules";

type Props = { tenant: string; slug: string };

/**
 * A aba Regras: primeiro "Como funciona", o texto que sai da configuração do
 * torneio; embaixo, as regras adicionais que a organização escreve (texto
 * simples, com as quebras de linha), no idioma de quem vê (sem tradução, o
 * original). Para a organização, até o torneio encerrar, a seção das
 * adicionais aparece sempre, com "Editar" (ou o convite a escrever) ali
 * mesmo; para os outros, só quando há texto: sem ele, o automático basta.
 * Fica em `<Suspense>`.
 */
export async function TournamentRules({ tenant, slug }: Props) {
  const [t, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { me, tournament } = loaded;
  const source = rulesSource(tournament, locale);
  const editable = me.role !== "player" && textsEditable(tournament.status);
  return (
    <Stack gap={7}>
      <TournamentAutoRules tournament={tournament} editable={editable} />
      {(editable || source.rules) && (
        <Section title={t("extraRules")}>
          {editable ? (
            <RulesEditor
              tenant={tenant}
              tournament={tournament.slug}
              locale={source.locale}
              rules={source.rules}
              translation={source.translation}
            />
          ) : (
            <UserText>{source.rules}</UserText>
          )}
        </Section>
      )}
    </Stack>
  );
}
