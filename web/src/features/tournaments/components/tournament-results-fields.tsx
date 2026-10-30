import { ScreenshotsRequiredField } from "@/features/screenshots/components/screenshots-required-field";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { TournamentContestField } from "./tournament-contest-field";

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Encerrado: nem a janela de contestação nem a captura de tela obrigatória mudam. */
  finished: boolean;
};

/**
 * Resultados: a janela de contestação e a captura de tela obrigatória (só
 * com capturas de tela na API). Ambas valem para os próximos envios e têm
 * padrão.
 */
export function TournamentResultsFields({
  values,
  onChange,
  errors,
  finished,
}: Props) {
  return (
    <>
      <TournamentContestField
        value={values.contestMinutes}
        onChange={(contestMinutes) => onChange({ contestMinutes })}
        error={errors.contestMinutes}
        disabled={finished}
      />
      <ScreenshotsRequiredField
        checked={values.screenshotsRequired}
        onChange={(screenshotsRequired) => onChange({ screenshotsRequired })}
        disabled={finished}
      />
    </>
  );
}
