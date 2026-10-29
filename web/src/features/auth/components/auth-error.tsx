import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/**
 * Mostra o erro com que o callback do login pelo provedor (`OAuthCallback`)
 * volta, em `?auth_error=`.
 */
export async function AuthError({ searchParams }: Props) {
  const { auth_error } = await searchParams;
  if (typeof auth_error !== "string") return null;

  return <Notice kind="error">{await apiErrorMessage(auth_error)}</Notice>;
}
