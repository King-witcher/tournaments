import { useTranslations } from "next-intl";
import { useId, useState } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { FormGroup } from "@/shared/ui/carbon/form";
import { ErrorText } from "@/shared/ui/error-text";
import {
  longestSeries,
  poolOf,
  type TournamentValues,
} from "../lib/form-values";
import { forgetPool, rememberPool } from "../lib/remembered-pool";
import styles from "./map-pool-field.module.scss";
import { MapPoolGrid } from "./map-pool-grid";

type Props = {
  tenant: string;
  values: Pick<TournamentValues, "mapPool" | "stages">;
  onChange: (patch: Partial<TournamentValues>) => void;
  error: string | undefined;
  /** Os mapas ativos (`GET /maps`). */
  maps: components["schemas"]["Map"][];
  /** O torneio já começou: só os mapas do pool, sem mudar. */
  readOnly: boolean;
  /**
   * O pool salvo, que divide os mapas em "no pool" e "fora do pool"; sem ele
   * (nada salvo ainda), o pool de quando o campo apareceu.
   */
  savedPool?: string[];
};

/**
 * O pool de mapas do torneio. Cada marca ou desmarca fica lembrada neste
 * navegador para a comunidade (o próximo torneio novo começa com ela);
 * "Usar o pool padrão" aparece quando o pool é outro e volta ao padrão,
 * esquecendo o lembrado. A divisão entre os do pool e os de fora segue o
 * salvo (`savedPool`): marcar ou desmarcar não muda o mapa de parte, e o que
 * entra fica marcado embaixo e o que sai desmarcado em cima, até salvar. Os
 * mapas de fora começam atrás de um botão (eram dois terços do grupo aberto)
 * e aparecem sozinhos quando o pool não basta ou quando algum deles está
 * marcado. Só para leitura, só os do pool.
 */
export function MapPoolField({
  tenant,
  values,
  onChange,
  error,
  maps,
  readOnly,
  savedPool,
}: Props) {
  const t = useTranslations("TournamentForm.veto");
  const headingId = useId();
  const outHeadingId = useId();
  const [outOpen, setOutOpen] = useState(false);
  const pool = poolOf(values, maps);
  const [initialPool] = useState(pool);
  const base = savedPool ?? initialPool;
  const min = longestSeries(values);
  const standard = maps.filter((m) => m.default_pool).map((m) => m.slug);
  const isStandard =
    pool.length === standard.length && standard.every((s) => pool.includes(s));
  const outShown =
    outOpen ||
    pool.length < min ||
    error !== undefined ||
    pool.some((s) => !base.includes(s));

  const toggle = (slug: string, on: boolean) => {
    const mapPool = on
      ? maps
          .filter((m) => pool.includes(m.slug) || m.slug === slug)
          .map((m) => m.slug)
      : pool.filter((s) => s !== slug);
    onChange({ mapPool });
    rememberPool(tenant, mapPool);
  };
  const showOut = () => {
    setOutOpen(true);
    // O botão some: o foco vai para o subtítulo dos de fora.
    focusIfLost(outHeadingId);
  };
  const resetToStandard = () => {
    onChange({ mapPool: standard });
    forgetPool(tenant);
    // O botão some: o foco vai para o subtítulo dos escolhidos.
    focusIfLost(headingId);
  };

  return (
    <FormGroup legendText={t("mapPool")} invalid={error !== undefined}>
      <div className={styles.hint}>
        <p className={styles.text}>{t("mapPoolHint", { min })}</p>
        {!isStandard && !readOnly && (
          <Button kind="ghost" size="sm" onClick={resetToStandard}>
            {t("useDefaultPool")}
          </Button>
        )}
      </div>
      <MapPoolGrid
        maps={maps}
        pool={pool}
        base={base}
        readOnly={readOnly}
        onToggle={toggle}
        headingId={headingId}
        outHeadingId={outHeadingId}
        outShown={outShown}
        onShowOut={showOut}
      />
      {error && <ErrorText>{error}</ErrorText>}
    </FormGroup>
  );
}
