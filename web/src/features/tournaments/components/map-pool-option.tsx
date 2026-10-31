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
 * Um mapa no pool do formulário: a imagem (em cinza fora do pool) e, embaixo,
 * a caixa com o nome. A imagem é um segundo rótulo da caixa, então clicar
 * nela também marca.
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
    <div className={styles.option}>
      <label
        htmlFor={id}
        className={cn(styles.image, disabled && styles.disabled)}
      >
        <MapThumbnail slug={map.slug} out={!checked} />
      </label>
      <Checkbox
        id={id}
        name={name}
        labelText={map.name}
        checked={checked}
        disabled={disabled}
        onChange={onChange}
      />
    </div>
  );
}
