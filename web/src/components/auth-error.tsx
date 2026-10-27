import { apiErrorMessage } from "@/lib/api/errors";

type Props = {
  searchParams: Promise<Record<string, string | string[] | undefined>>;
};

/** Mostra o erro que o callback do login devolve em `?auth_error=`. */
export async function AuthError({ searchParams }: Props) {
  const { auth_error } = await searchParams;
  if (typeof auth_error !== "string") return null;

  return (
    <p
      role="alert"
      className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm"
    >
      {await apiErrorMessage(auth_error)}
    </p>
  );
}
