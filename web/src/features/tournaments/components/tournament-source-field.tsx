import { useLocale, useTranslations } from "next-intl";
import { memo, useRef, useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { InlineLoading } from "@/shared/ui/carbon/inline-loading";
import { ComboBox } from "@/shared/ui/combo-box";
import { localizedName } from "../lib/localize";

type Schemas = components["schemas"];
type Summary = Schemas["TournamentSummary"];

type Props = {
  tenant: string;
  /** Os torneios da comunidade, do mais novo ao mais antigo. */
  tournaments: Summary[];
  /** O torneio com que o formulário abriu (`?from=`). */
  initial: string | undefined;
  /** Aplica a origem escolhida (`undefined`: nenhuma); a mesma função em todo render. */
  onCopy: (source: Schemas["Tournament"] | undefined) => void;
};

/** O `?from=` acompanha a escolha, para recarregar a página voltar a ela. */
function syncUrl(slug: string | undefined) {
  const url = new URL(window.location.href);
  if (slug) url.searchParams.set("from", slug);
  else url.searchParams.delete("from");
  window.history.replaceState(null, "", url);
}

/**
 * "Copiar de", no topo da criação: escolhe o torneio da comunidade de onde
 * vem a configuração. Busca o torneio escolhido e só então o aplica; se a
 * busca falha, a escolha volta à anterior, que é a que está no formulário.
 */
export const TournamentSourceField = memo(function TournamentSourceField({
  tenant,
  tournaments,
  initial,
  onCopy,
}: Props) {
  const t = useTranslations("TournamentForm.source");
  const tCommon = useTranslations("Common");
  const locale = useLocale();
  const { run, pending, error } = useApiAction();
  const [picked, setPicked] = useState(
    () => tournaments.find((x) => x.slug === initial) ?? null,
  );
  // O que está aplicado no formulário: o Carbon chama `onChange` também
  // quando a escolha volta à anterior, e aí não há o que buscar.
  const applied = useRef(picked);
  const label = (item: Summary | null) =>
    item ? localizedName(item, locale) : "";

  const select = (item: Summary | null) => {
    setPicked(item);
    if (item?.slug === applied.current?.slug) return;
    if (!item) {
      applied.current = null;
      onCopy(undefined);
      syncUrl(undefined);
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
        syncUrl(item.slug);
      },
      () => setPicked(applied.current),
    );
  };

  return (
    <ComboBox
      titleText={t("label")}
      placeholder={t("none")}
      items={tournaments}
      itemToString={label}
      selectedItem={picked}
      onChange={({ selectedItem }) => select(selectedItem ?? null)}
      // Filtra pelo que foi digitado; com o nome escolhido no campo, mostra todos.
      shouldFilterItem={({ item, inputValue }) =>
        !inputValue ||
        inputValue === label(picked) ||
        label(item).toLowerCase().includes(inputValue.toLowerCase())
      }
      readOnly={pending}
      helperText={
        pending ? <InlineLoading description={tCommon("loading")} /> : undefined
      }
      invalid={error !== undefined}
      invalidText={error}
    />
  );
});
