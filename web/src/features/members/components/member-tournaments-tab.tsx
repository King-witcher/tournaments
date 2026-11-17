import { getCurrentUser } from "@/features/auth/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { getMember, getMemberTournaments } from "../server";
import { MemberTournaments } from "./member-tournaments";

type Props = {
  tenant: string;
  /** O `user_id` do endereço, ainda como texto. */
  user: string;
};

/**
 * A aba Torneios: busca os torneios do membro e monta a tabela. Sem o
 * membro, nada (a casca já mostra o erro). Fica em `<Suspense>`.
 */
export async function MemberTournamentsTab({ tenant, user }: Props) {
  const member = await getMember(tenant, user);
  if (!member.ok) return null;
  const [loaded, me] = await Promise.all([
    getMemberTournaments(tenant, member.data.user_id),
    getCurrentUser(),
  ]);
  if (!loaded.ok) {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  return (
    <MemberTournaments
      tenant={tenant}
      tournaments={loaded.data}
      nickname={member.data.nickname}
      isYou={me?.id === member.data.user_id}
    />
  );
}
