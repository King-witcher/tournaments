import { useTranslations } from "next-intl";

import { Checkbox } from "@/shared/ui/checkbox";

type Props = {
  checked: boolean;
  onChange: (checkIn: boolean) => void;
  /** O torneio já começou: só para leitura. */
  readOnly: boolean;
  /** TDM: o check-in decide quem entra num time, não na chave. */
  teams?: boolean;
};

/** Se o torneio tem check-in, que a organização abre pelo botão. */
export function TournamentCheckInField({
  checked,
  onChange,
  readOnly,
  teams = false,
}: Props) {
  const t = useTranslations("TournamentForm");
  const hint = checked ? "checkInHint" : "checkInOffHint";

  return (
    <Checkbox
      labelText={t("checkIn")}
      helperText={t(teams ? `${hint}Tdm` : hint)}
      name="checkIn"
      checked={checked}
      readOnly={readOnly}
      onChange={onChange}
    />
  );
}
