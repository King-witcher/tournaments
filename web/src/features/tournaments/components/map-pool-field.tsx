import { FormGroup } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useId } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
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
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  error: string | undefined;
  /** Os mapas ativos (`GET /maps`). */
  maps: components["schemas"]["Map"][];
  disabled: boolean;
};

/**
 * O pool de mapas do torneio. Cada marca ou desmarca fica lembrada neste
 * navegador para a comunidade (o próximo torneio novo começa com ela);
 * "Usar o pool padrão" aparece quando o pool é outro e volta ao padrão,
 * esquecendo o lembrado.
 */
export function MapPoolField({
  tenant,
  values,
  onChange,
  error,
  maps,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm.veto");
  const headingId = useId();
  const pool = poolOf(values, maps);
  const standard = maps.filter((m) => m.default_pool).map((m) => m.slug);
  const isStandard =
    pool.length === standard.length && standard.every((s) => pool.includes(s));

  const toggle = (slug: string, on: boolean) => {
    const mapPool = on
      ? maps
          .filter((m) => pool.includes(m.slug) || m.slug === slug)
          .map((m) => m.slug)
      : pool.filter((s) => s !== slug);
    onChange({ mapPool });
    rememberPool(tenant, mapPool);
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
        <p className={styles.text}>
          {t("mapPoolHint", { min: longestSeries(values) })}
        </p>
        {!isStandard && !disabled && (
          <Button kind="ghost" size="sm" onClick={resetToStandard}>
            {t("useDefaultPool")}
          </Button>
        )}
      </div>
      <MapPoolGrid
        maps={maps}
        pool={pool}
        disabled={disabled}
        onToggle={toggle}
        headingId={headingId}
      />
      {error && <ErrorText>{error}</ErrorText>}
    </FormGroup>
  );
}
