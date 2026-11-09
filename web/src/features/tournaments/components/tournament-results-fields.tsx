import { memo } from "react";
import { ScreenshotsRequiredField } from "@/features/screenshots/components/screenshots-required-field";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { TournamentContestField } from "./tournament-contest-field";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["contestMinutes", "screenshotsRequired"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Encerrado: a janela de contestação e a captura de tela só para leitura. */
  readOnly: boolean;
  /** Se esta API tem capturas de tela. */
  screenshots: boolean;
};

/**
 * Resultados: a janela de contestação e a captura de tela obrigatória (só com
 * capturas de tela na API). Ambas valem para os próximos envios e têm padrão.
 */
export const TournamentResultsFields = memo(function TournamentResultsFields({
  values,
  onChange,
  errors,
  readOnly,
  screenshots,
}: Props) {
  return (
    <>
      <TournamentContestField
        value={values.contestMinutes}
        onChange={(contestMinutes) => onChange({ contestMinutes })}
        error={errors.contestMinutes}
        readOnly={readOnly}
      />
      {screenshots && (
        <ScreenshotsRequiredField
          checked={values.screenshotsRequired}
          onChange={(screenshotsRequired) => onChange({ screenshotsRequired })}
          readOnly={readOnly}
        />
      )}
    </>
  );
}, sameValues(KEYS));
