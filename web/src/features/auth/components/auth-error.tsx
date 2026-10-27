import { apiErrorMessage } from "@/shared/api/errors";
import { Alert } from "@/shared/ui/alert";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/** Mostra o erro que o callback do login devolve em `?auth_error=`. */
export async function AuthError({ searchParams }: Props) {
  const { auth_error } = await searchParams;
  if (typeof auth_error !== "string") return null;

  return <Alert>{await apiErrorMessage(auth_error)}</Alert>;
}
