import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";

/**
 * Bloco cinza pulsando no lugar de um conteúdo que ainda vai chegar: ocupa o
 * espaço dele (a página não pula quando chega) e o pulso diz que não acabou.
 */
export function Skeleton({ className }: { className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        "block animate-pulse rounded-md bg-black/10 dark:bg-white/10",
        className,
      )}
    />
  );
}

/**
 * Seção carregando: o texto (lido também por leitor de tela) e linhas com a
 * altura de um item de lista.
 */
export function LoadingSection({
  label,
  rows = 3,
}: {
  /** O que está carregando; sem ele, "Carregando…". */
  label?: string;
  rows?: number;
}) {
  const t = useTranslations("Common");
  return (
    <section role="status" className="flex flex-col gap-3">
      <p className="text-sm text-muted">{label ?? t("loading")}</p>
      <ul className="flex flex-col gap-2">
        {Array.from({ length: rows }, (_, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, sem identidade.
          <li key={i}>
            <Skeleton className="h-11" />
          </li>
        ))}
      </ul>
    </section>
  );
}

/** Página carregando: lugar do título e de uma seção. */
export function PageSkeleton({ label }: { label?: string }) {
  return (
    <div className="flex flex-col gap-8">
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-9 w-2/3" />
      </div>
      <LoadingSection label={label} rows={4} />
    </div>
  );
}
