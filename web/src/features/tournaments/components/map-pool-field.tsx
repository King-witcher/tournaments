import { useTranslations } from "next-intl";
import { useState } from "react";

import type { components } from "@/shared/api/schema";
import { FormGroup } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { ErrorText } from "@/shared/ui/error-text";
import { FormRow } from "@/shared/ui/form-row";
import { Select } from "@/shared/ui/select";
import {
  isTdm,
  longestSeries,
  poolOf,
  type TournamentValues,
} from "../lib/form-values";
import {
  isDefaultPool,
  type MapPoolChoice,
  mapPoolChoiceOf,
  mapPoolTemplates,
} from "../lib/map-pool-templates";
import { forgetPool, rememberPool } from "../lib/remembered-pool";
import styles from "./map-pool-field.module.scss";
import { MapPoolPicker } from "./map-pool-picker";

type Props = {
  tenant: string;
  values: Pick<TournamentValues, "mapPool" | "stages" | "mode">;
  onChange: (patch: Partial<TournamentValues>) => void;
  error: string | undefined;
  /** Os mapas ativos (`GET /maps`). */
  maps: components["schemas"]["Map"][];
  /** O torneio já começou: só os mapas do pool, sem mudar. */
  readOnly: boolean;
  /**
   * O pool salvo, que divide os mapas em "no pool" e "fora do pool"; sem ele
   * (nada salvo ainda), o pool de quando a grade apareceu.
   */
  savedPool?: string[];
};

/**
 * O pool de mapas do torneio: um `Select` de modelo (`lib/map-pool-templates`)
 * e a grade de imagens, sempre à vista, porque identificar um mapa pela imagem
 * é mais rápido que ler a lista. Escolher um modelo marca os mapas dele e
 * reparte a grade entre os do modelo e os de fora; marcar um mapa faz o `Select`
 * seguir o pool (Personalizado, ou o modelo que ele voltou a formar). `picked`
 * guarda a escolha explícita: Personalizado continua nele com o pool igual a um
 * modelo, até marcar um mapa. O pool escolhido fica lembrado neste navegador
 * para a comunidade (o próximo torneio novo começa com ele); o modelo igual ao
 * padrão esquece o lembrado. Só para leitura, só os mapas do pool, sem o
 * `Select`.
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
  const [picked, setPicked] = useState<MapPoolChoice>();
  const [templatePool, setTemplatePool] = useState<string[]>();
  const pool = poolOf(values, maps);
  const min = longestSeries(values);
  const templates = mapPoolTemplates(maps, isTdm(values));
  const choice =
    picked === "custom" ? "custom" : mapPoolChoiceOf(pool, templates, picked);

  const choose = (next: MapPoolChoice) => {
    setPicked(next);
    const template = templates.find((item) => item.id === next);
    if (!template) return; // Personalizado parte do pool de agora.
    setTemplatePool(template.pool);
    onChange({ mapPool: template.pool });
    if (isDefaultPool(template.pool, maps)) forgetPool(tenant);
    else rememberPool(tenant, template.pool);
  };
  const toggled = (mapPool: string[]) => {
    setPicked(undefined);
    onChange({ mapPool });
  };

  return (
    <FormGroup legendText={t("mapPool")} invalid={error !== undefined}>
      <Stack gap={5}>
        <p className={styles.hint}>{t("mapPoolHint", { min })}</p>
        {!readOnly && (
          <FormRow columns={4}>
            <Select
              labelText={t("poolTemplate")}
              value={choice}
              onChange={(e) => choose(e.target.value as MapPoolChoice)}
            >
              {templates.map((item) => (
                <option key={item.id} value={item.id}>
                  {t(`poolTemplates.${item.id}`)}
                </option>
              ))}
              <option value="custom">{t("poolTemplates.custom")}</option>
            </Select>
          </FormRow>
        )}
        <MapPoolPicker
          tenant={tenant}
          maps={maps}
          pool={pool}
          min={min}
          hasError={error !== undefined}
          readOnly={readOnly}
          savedPool={savedPool}
          templatePool={templatePool}
          onChange={toggled}
        />
        {error && <ErrorText>{error}</ErrorText>}
      </Stack>
    </FormGroup>
  );
}
