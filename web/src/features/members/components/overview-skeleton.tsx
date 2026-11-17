import { useTranslations } from "next-intl";

import { Column, Grid } from "@/shared/ui/carbon/grid";
import { LineChartSkeleton } from "@/shared/ui/line-chart";
import { Panel } from "@/shared/ui/panel";
import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";
import styles from "./member-overview.module.scss";

/**
 * A Visão geral carregando, na forma dela: Rating e Carreira lado a lado, o
 * gráfico na altura dele, os mapas em tiles e as últimas séries.
 */
export function OverviewSkeleton() {
  const t = useTranslations("Member.loadingTab");
  return (
    <TabLoading label={t("overview")}>
      <Grid>
        <Column sm={4} md={8} lg={8}>
          <Panel>
            <SectionSkeleton rows={4} />
          </Panel>
        </Column>
        <Column sm={4} md={8} lg={8} className={styles.career}>
          <Panel>
            <SectionSkeleton rows={4} />
          </Panel>
        </Column>
      </Grid>
      <Panel>
        <SectionSkeleton rows={0} />
        <LineChartSkeleton />
      </Panel>
      <Panel>
        <SectionSkeleton tiles={3} />
      </Panel>
      <TableSkeleton columns={4} rows={5} />
    </TabLoading>
  );
}
