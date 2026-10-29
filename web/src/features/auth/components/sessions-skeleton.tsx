import { FormSkeleton } from "@/shared/ui/skeleton";

/** A aba Sessões carregando: o título, a frase e as linhas da lista. */
export function SessionsSkeleton({ label }: { label?: string | false }) {
  return <FormSkeleton heading text fields={2} label={label} />;
}
