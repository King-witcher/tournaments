import { useLocale, useTranslations } from "next-intl";
import { memo, useRef, useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { InlineLoading } from "@/shared/ui/carbon/inline-loading";
import { ComboBox } from "@/shared/ui/combo-box";
import { localizedName } from "../lib/localize";

type Schemas = components["schemas"];

/** Um torneio da lista: o que o campo mostra dele. */
export type SourceItem = Pick<
  Schemas["TournamentSummary"],
  "slug" | "name" | "translations"
>;

type Props = {
  tenant: string;
  /** Os torneios da comunidade, do mais novo ao mais antigo. */
  tournaments: SourceItem[];
  /** A lista ainda está chegando: o campo espera, só para leitura. */
  loading?: boolean;
  /** O torneio de onde começa ao abrir ("Copiar torneio", `?from=`). */
  initial: string | undefined;
  /** Aplica a origem escolhida (`undefined`: o padrão); a mesma função em todo render. */
  onCopy: (source: Schemas["Tournament"] | undefined) => void;
  /** Na página: o `?from=` acompanha a escolha, para recarregar voltar a ela. */
  syncUrl?: boolean;
};

function replaceFrom(slug: string | undefined) {
  const url = new URL(window.location.href);
  if (slug) url.searchParams.set("from", slug);
  else url.searchParams.delete("from");
  window.history.replaceState(null, "", url);
}

/**
 * "Começar de", na criação: vazio, a configuração padrão; ou um torneio da
 * comunidade, de onde vem a configuração. Busca o torneio escolhido e só
 * então o aplica; se a busca falha, a escolha volta à anterior, que é a que
 * vale.
 */
export const TournamentSourceField = memo(function TournamentSourceField({
  tenant,
  tournaments,
  loading = false,
  initial,
  onCopy,
  syncUrl = false,
}: Props) {
  const t = useTranslations("TournamentForm.source");
  const tCommon = useTranslations("Common");
  const locale = useLocale();
  const { run, pending, error } = useApiAction();
  const [picked, setPicked] = useState(
    () => tournaments.find((x) => x.slug === initial) ?? null,
  );
  // A lista chegou: a escolha passa a ser o item dela. Sem remontar o campo,
  // que é o primeiro do Modal e pode estar com o foco.
  const [waiting, setWaiting] = useState(loading);
  if (waiting && !loading) {
    setWaiting(false);
    setPicked(
      tournaments.find((x) => x.slug === (picked?.slug ?? initial)) ?? null,
    );
  }
  // O que está aplicado: o Carbon chama `onChange` também quando a escolha
  // volta à anterior, e aí não há o que buscar.
  const applied = useRef(picked);
  const label = (item: SourceItem | null) =>
    item ? localizedName(item, locale) : "";

  const select = (item: SourceItem | null) => {
    setPicked(item);
    if (item?.slug === applied.current?.slug) return;
    if (!item) {
      applied.current = null;
      onCopy(undefined);
      if (syncUrl) replaceFrom(undefined);
      return;
    }
    run(
      () =>
        api.GET("/tenants/{slug}/tournaments/{tournament}", {
          params: { path: { slug: tenant, tournament: item.slug } },
        }),
      (source) => {
        applied.current = item;
        onCopy(source);
        if (syncUrl) replaceFrom(item.slug);
      },
      () => setPicked(applied.current),
    );
  };
  const busy = pending || loading;

  return (
    <ComboBox
      titleText={t("label")}
      placeholder={t("none")}
      items={tournaments}
      itemToString={label}
      selectedItem={picked}
      // A lista flutua por cima: no Modal, o corpo dele a cortaria.
      autoAlign
      onChange={({ selectedItem }) => select(selectedItem ?? null)}
      // Filtra pelo que foi digitado; com o nome escolhido no campo, mostra todos.
      shouldFilterItem={({ item, inputValue }) =>
        !inputValue ||
        inputValue === label(picked) ||
        label(item).toLowerCase().includes(inputValue.toLowerCase())
      }
      readOnly={busy}
      helperText={
        busy ? <InlineLoading description={tCommon("loading")} /> : undefined
      }
      invalid={error !== undefined}
      invalidText={error}
    />
  );
});
