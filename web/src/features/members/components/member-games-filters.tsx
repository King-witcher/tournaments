"use client";

import { useLocale, useTranslations } from "next-intl";
import { useMemo, useOptimistic } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ComboBox } from "@/shared/ui/combo-box";
import { FormRow } from "@/shared/ui/form-row";
import { useSearchTransition } from "../hooks/use-search-transition";
import { type GamesQuery, gamesHref } from "../lib/games-href";
import styles from "./member-games-filters.module.scss";

type Props = {
  /** A página da pessoa. */
  base: string;
  filters: components["schemas"]["GameFilters"];
  query: GamesQuery;
};

type Item = { id: string; label: string };

/**
 * Os filtros da aba Partidas, acima da tabela: torneio, mapa e adversário
 * (listas que crescem: digita e filtra). Escolher muda o endereço e volta à
 * página 1; limpar o campo tira o filtro, e "Limpar filtros" tira todos. O
 * campo mostra a escolha na hora, e a tabela fica em esqueleto até chegar
 * (`SearchTransition`). O Carbon repete o `onChange` quando o item escolhido
 * muda por fora: o mesmo endereço não navega de novo.
 */
export function MemberGamesFilters({ base, filters, query }: Props) {
  const t = useTranslations("Member.games");
  const tMember = useTranslations("Member");
  const locale = useLocale();
  const { navigate } = useSearchTransition();
  const [shown, setShown] = useOptimistic(query);
  const go = (change: GamesQuery) => {
    const next = { ...shown, ...change, page: undefined };
    const href = gamesHref(base, next);
    if (href === gamesHref(base, { ...shown, page: undefined })) return;
    navigate(href, () => setShown(next));
  };

  // Os mesmos objetos a cada render: o Carbon compara o item escolhido.
  const tournaments = useMemo<Item[]>(
    () =>
      filters.tournaments.map((tr) => ({
        id: tr.slug,
        label: localizedName(tr, locale),
      })),
    [filters.tournaments, locale],
  );
  const maps = useMemo<Item[]>(
    () => filters.maps.map((m) => ({ id: m.slug, label: m.name })),
    [filters.maps],
  );
  const opponents = useMemo<Item[]>(
    () =>
      filters.opponents.map((p) => {
        const name = shownNickname(p.nickname, tMember("deletedAccount"));
        return {
          id: String(p.user_id),
          label:
            p.active || p.nickname === "" ? name : tMember("left", { name }),
        };
      }),
    [filters.opponents, tMember],
  );
  const find = (items: Item[], id: string | undefined) =>
    items.find((item) => item.id === id) ?? null;
  const filtered =
    shown.tournament !== undefined ||
    shown.map !== undefined ||
    shown.opponent !== undefined;

  return (
    <div className={styles.filters}>
      <FormRow columns={3}>
        <ComboBox<Item>
          titleText={t("tournament")}
          placeholder={t("any")}
          items={tournaments}
          selectedItem={find(tournaments, shown.tournament)}
          itemToString={(item) => item?.label ?? ""}
          onChange={({ selectedItem }) => go({ tournament: selectedItem?.id })}
        />
        <ComboBox<Item>
          titleText={t("map")}
          placeholder={t("any")}
          items={maps}
          selectedItem={find(maps, shown.map)}
          itemToString={(item) => item?.label ?? ""}
          onChange={({ selectedItem }) => go({ map: selectedItem?.id })}
        />
        <ComboBox<Item>
          titleText={t("opponent")}
          placeholder={t("any")}
          items={opponents}
          selectedItem={find(opponents, shown.opponent?.toString())}
          itemToString={(item) => item?.label ?? ""}
          onChange={({ selectedItem }) =>
            go({
              opponent: selectedItem ? Number(selectedItem.id) : undefined,
            })
          }
        />
      </FormRow>
      {filtered && (
        <div>
          <Button
            kind="ghost"
            onClick={() =>
              go({ tournament: undefined, map: undefined, opponent: undefined })
            }
          >
            {t("clear")}
          </Button>
        </div>
      )}
    </div>
  );
}
