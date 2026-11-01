import { useTranslations } from "next-intl";

import { MapGrid } from "@/features/maps/components/map-grid";
import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import styles from "./map-pool-grid.module.scss";
import { MapPoolOption } from "./map-pool-option";

type Props = {
  /** Os mapas ativos, na ordem da API (o padrão primeiro). */
  maps: components["schemas"]["Map"][];
  /** Os slugs escolhidos. */
  pool: string[];
  disabled: boolean;
  onToggle: (slug: string, on: boolean) => void;
  /** O `id` do subtítulo dos escolhidos, alvo do foco quando o controle que a pessoa usou some. */
  headingId: string;
  /** O `id` do subtítulo dos de fora, alvo do foco depois de "Mostrar". */
  outHeadingId: string;
  /** Os de fora à vista; sem isso, no lugar deles, o botão que os mostra. */
  outShown: boolean;
  onShowOut: () => void;
};

/**
 * O pool em duas partes, os escolhidos e depois os de fora, cada uma com o
 * subtítulo e a contagem; marcar ou desmarcar muda o mapa de parte na hora,
 * para um de fora não passar despercebido no meio dos escolhidos. Uma grade
 * só, com os subtítulos como itens dela: o React move a caixa em vez de
 * recriá-la, e o foco do teclado continua no mapa.
 */
export function MapPoolGrid({
  maps,
  pool,
  disabled,
  onToggle,
  headingId,
  outHeadingId,
  outShown,
  onShowOut,
}: Props) {
  const t = useTranslations("TournamentForm.veto");
  const parts = [
    {
      key: "in",
      title: "poolIn",
      headingId,
      maps: maps.filter((m) => pool.includes(m.slug)),
    },
    {
      key: "out",
      title: "poolOut",
      headingId: outHeadingId,
      maps: maps.filter((m) => !pool.includes(m.slug)),
    },
  ] as const;
  // O foco do erro vai para a primeira caixa.
  const first = parts.find((p) => p.maps.length > 0)?.maps[0]?.slug;

  return (
    <MapGrid>
      {parts.flatMap((p) => {
        if (p.maps.length === 0) return [];
        if (p.key === "out" && !outShown) {
          return [
            <div key={p.key} className={styles.part}>
              <Button kind="ghost" size="sm" onClick={onShowOut}>
                {t("showPoolOut", { count: p.maps.length })}
              </Button>
            </div>,
          ];
        }
        return [
          <p key={p.key} id={p.headingId} tabIndex={-1} className={styles.part}>
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
        ];
      })}
    </MapGrid>
  );
}
