"use client";

import { CheckmarkOutline } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Tab, TabPanel, TabPanels, Tabs } from "@/shared/ui/carbon/tabs";
import { FormModal } from "@/shared/ui/form-modal";
import { TabList } from "@/shared/ui/tab-list";
import { type Tie, type TieDraft, tieKey } from "../lib/ties";
import { TieForm, type TieStatus } from "./tie-form";
import styles from "./ties-modal.module.scss";

type Props = {
  tenant: string;
  slug: string;
  /** Os empates pendentes agora (mudam com o tempo real). */
  ties: Tie[];
  /** É a última fase: o último empate decidido encerra o torneio. */
  lastStage: boolean;
  onClose: () => void;
};

/**
 * Os empates do fim da fase, um por vez, em abas de linha: a pessoa vai e volta
 * entre eles na ordem que quiser (o `ProgressIndicator` do Carbon é para
 * processo linear e marca como feito tudo antes do passo atual), e cada um se
 * salva sozinho. O que ainda não foi salvo fica guardado aqui enquanto o Modal
 * está aberto. Uma aba nunca some: a do empate salvo aqui ou decidido por outra
 * pessoa da organização (o tempo real tira dos `ties`) ganha o check e mostra
 * o estado, e o resto continua como estava. O último salvo fecha o Modal.
 */
export function TiesModal({ tenant, slug, ties, lastStage, onClose }: Props) {
  const t = useTranslations("Bracket.ties");
  const tCommon = useTranslations("Common");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  // Todos os empates vistos desde que abriu: a lista só cresce.
  const [seen, setSeen] = useState(ties);
  const [selected, setSelected] = useState(ties[0]?.key);
  const [drafts, setDrafts] = useState<Record<string, TieDraft>>({});
  const [saved, setSaved] = useState<Record<string, TieDraft>>({});
  const [stale, setStale] = useState<string[]>([]);
  const [failed, setFailed] = useState<string>();

  const unseen = ties.filter((tie) => !seen.some((s) => s.key === tie.key));
  if (unseen.length > 0) setSeen([...seen, ...unseen]);

  const items = [...seen, ...unseen]
    .map((s) => ties.find((tie) => tie.key === s.key) ?? s)
    .sort((a, b) => a.position - b.position);
  const statusOf = (key: string): TieStatus => {
    if (saved[key]) return "saved";
    return stale.includes(key) || !ties.some((tie) => tie.key === key)
      ? "gone"
      : "pending";
  };
  const open = items.filter((tie) => statusOf(tie.key) === "pending");
  const current =
    items.find((tie) => tie.key === selected) ?? open[0] ?? items[0];
  if (!current) return null;

  const status = statusOf(current.key);
  const draftOf = (tie: Tie): TieDraft =>
    saved[tie.key] ??
    drafts[tie.key] ?? { order: tie.teams.map((team) => team.id), reason: "" };
  const draft = draftOf(current);

  function save() {
    const { key } = current;
    const { order, reason } = draft;
    setFailed(undefined);
    run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/ties", {
          params: { path: { slug: tenant, tournament: slug } },
          body: { teams: order, reason: reason.trim() || null },
        }),
      (bracket) => {
        setSaved((prev) => ({ ...prev, [key]: { order, reason } }));
        const next = bracket.pending_ties
          .map((tie) => tieKey(tie.team_ids))
          .find((k) => k !== key);
        if (next) setSelected(next);
        else onClose();
        router.refresh();
      },
      (code) => {
        // Outra pessoa decidiu (ou o resultado mudou) antes: a aba mostra que
        // não está mais pendente, e o refresh acerta o resto.
        if (code === "tie.not_pending") {
          setStale((prev) => [...prev, key]);
          router.refresh();
        } else {
          setFailed(key);
        }
      },
    );
  }

  const form = (
    <TieForm
      tie={current}
      status={status}
      draft={draft}
      ending={
        open.length === 1 && status === "pending"
          ? lastStage
            ? "tournament"
            : "stage"
          : undefined
      }
      onChange={(next) =>
        setDrafts((prev) => ({ ...prev, [current.key]: next }))
      }
    />
  );

  return (
    <FormModal
      title={
        items.length > 1
          ? t("title")
          : t("group", { position: current.position })
      }
      size="md"
      submitLabel={t("save")}
      cancelLabel={tCommon("close")}
      pending={pending}
      submitDisabled={status !== "pending"}
      error={failed === current.key ? error : undefined}
      onSubmit={save}
      onClose={onClose}
    >
      {items.length > 1 ? (
        <Tabs
          selectedIndex={items.indexOf(current)}
          onChange={({ selectedIndex }) =>
            setSelected(items[selectedIndex].key)
          }
        >
          <TabList aria-label={t("title")}>
            {items.map((tie) => {
              const done = statusOf(tie.key) !== "pending";
              return (
                <Tab
                  key={tie.key}
                  renderIcon={done ? CheckmarkOutline : undefined}
                  aria-label={
                    done ? t("tabDone", { position: tie.position }) : undefined
                  }
                  data-modal-primary-focus={
                    tie.key === current.key ? true : undefined
                  }
                >
                  {t("tab", { position: tie.position })}
                </Tab>
              );
            })}
          </TabList>
          <TabPanels>
            {items.map((tie) => (
              <TabPanel key={tie.key} className={styles.panel}>
                {tie.key === current.key ? form : null}
              </TabPanel>
            ))}
          </TabPanels>
        </Tabs>
      ) : (
        <div tabIndex={-1} data-modal-primary-focus>
          {form}
        </div>
      )}
    </FormModal>
  );
}
