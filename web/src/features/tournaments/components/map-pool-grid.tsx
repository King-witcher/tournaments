import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import styles from "./map-pool-grid.module.scss";
import { MapPoolOption } from "./map-pool-option";

type Props = {
  /** Os mapas ativos, na ordem da API (o padrão primeiro). */
  maps: components["schemas"]["Map"][];
  /** Os slugs escolhidos. */
  pool: string[];
  disabled: boolean;
  onToggle: (slug: string, on: boolean) => void;
};

/**
 * O pool em duas partes, os escolhidos e depois os de fora, cada uma com o
 * subtítulo e a contagem; marcar ou desmarcar muda o mapa de parte na hora,
 * para um de fora não passar despercebido no meio dos escolhidos. Uma grade
 * só, com os subtítulos como itens dela: o React move a caixa em vez de
 * recriá-la, e o foco do teclado continua no mapa.
 */
export function MapPoolGrid({ maps, pool, disabled, onToggle }: Props) {
  const t = useTranslations("TournamentForm.veto");
  const parts = [
    {
      key: "in",
      title: "poolIn",
      maps: maps.filter((m) => pool.includes(m.slug)),
    },
    {
      key: "out",
      title: "poolOut",
      maps: maps.filter((m) => !pool.includes(m.slug)),
    },
  ] as const;
  // O foco do erro vai para a primeira caixa.
  const first = parts.find((p) => p.maps.length > 0)?.maps[0]?.slug;

  return (
    <div className={styles.grid}>
      {parts.flatMap((p) =>
        p.maps.length === 0
          ? []
          : [
              <p key={p.key} className={styles.part}>
                {t(p.title, { count: p.maps.length })}
              </p>,
              ...p.maps.map((m) => (
                <MapPoolOption
                  key={m.slug}
                  map={m}
                  name={m.slug === first ? "mapPool" : undefined}
                  checked={p.key === "in"}
                  disabled={disabled}
                  onChange={(on) => onToggle(m.slug, on)}
                />
              )),
            ],
      )}
    </div>
  );
}
