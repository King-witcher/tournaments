import { FormSkeleton } from "@/shared/ui/skeleton";

/**
 * A aba Logins carregando, na forma dela: o bloco do email (título, frase,
 * campo, botão), o do Discord (título, frase, botão) e o da senha (título,
 * frase, campo, botão).
 */
export function LoginsSkeleton({ label }: { label?: string | false }) {
  return (
    <>
      <FormSkeleton heading text fields={1} label={label} />
      <FormSkeleton heading text fields={0} label={false} />
      <FormSkeleton heading text fields={1} label={false} />
    </>
  );
}
