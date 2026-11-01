import { useId } from "react";

import { MapThumbnail } from "@/features/maps/components/map-thumbnail";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Checkbox } from "@/shared/ui/checkbox";
import styles from "./map-pool-option.module.scss";

type Props = {
  map: components["schemas"]["Map"];
  checked: boolean;
  disabled: boolean;
  /** O nome do campo, só na primeira caixa: o foco do erro vai para ela. */
  name?: string;
  onChange: (checked: boolean) => void;
};

/**
 * Um mapa no pool do formulário, numa caixa como o tile selecionável do
 * Carbon: a imagem (em cinza fora do pool) e, embaixo, a caixa de seleção com
 * o nome; o escolhido ganha a borda de selecionado. A imagem é um segundo
 * rótulo da caixa, esticado sobre a caixa inteira, então clicar em qualquer
 * ponto marca. Não é o `SelectableTile` da `shared/ui`: ele põe a marca no
 * canto de cima, sobre a imagem, e reserva uma faixa à direita para ela.
 */
export function MapPoolOption({
  map,
  checked,
  disabled,
  name,
  onChange,
}: Props) {
  const id = useId();
  return (
    <div
      className={cn(
        styles.option,
        checked && styles.selected,
        disabled && styles.disabled,
      )}
    >
      <label htmlFor={id} className={styles.image}>
        <MapThumbnail slug={map.slug} out={!checked} />
      </label>
      <div className={styles.check}>
        <Checkbox
          id={id}
          name={name}
          labelText={map.name}
          checked={checked}
          disabled={disabled}
          onChange={onChange}
        />
      </div>
    </div>
  );
}
