import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { StatusTag } from "@/shared/ui/status-tag";
import { useTeamName } from "../hooks/use-team-name";
import { soloMember } from "../lib/team-name";
import styles from "./seeding-row.module.scss";

type Props = {
  /** Seed que fica se a ordem for salva (1 = melhor). */
  position: number;
  /** Sem número quando a ordem ainda não vale (ninguém tem seed). */
  numbered: boolean;
  team: components["schemas"]["Team"];
  /** Marca quem está sem seed (quando só alguns estão). */
  showNoSeed: boolean;
  /** O rating na comunidade ("Rating 1569", "Sem rating"); sem nada quando ninguém da lista tem. */
  ratingText?: string;
};

/** Uma linha da ordem da chave: a posição, o time (no duelo, o jogador) e o rating. */
export function SeedingRow({
  position,
  numbered,
  team,
  showNoSeed,
  ratingText,
}: Props) {
  const t = useTranslations("Participants.seeding");
  const tMember = useTranslations("Member");
  const name = useTeamName()(team);
  const solo = soloMember(team);

  return (
    <div className={styles.row}>
      {numbered ? (
        <span className={styles.position}>{position}</span>
      ) : (
        <span className={styles.position} aria-hidden="true">
          –
        </span>
      )}
      <span className={styles.name}>
        <Avatar src={solo?.avatar_url} name={name} />
        <span className={styles.label}>
          {name}
          {/* No TDM, quem está no time, para reconhecer o "Time de {capitão}". */}
          {!solo && (
            <span className={styles.members}>
              {team.members
                .map((m) =>
                  shownNickname(m.nickname, tMember("deletedAccount")),
                )
                .join(", ")}
            </span>
          )}
        </span>
      </span>
      {ratingText && <span className={styles.rating}>{ratingText}</span>}
      {showNoSeed && team.seed == null && (
        <StatusTag status="neutral" size="sm">
          {t("noSeed")}
        </StatusTag>
      )}
    </div>
  );
}
