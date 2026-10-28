import { loadBracket } from "@/features/matches/server";
import { UndoStartButton } from "./undo-start-button";

type Props = {
  tenant: string;
  slug: string;
  /** Para onde volta: o texto da confirmação. */
  message: string;
};

/**
 * Desfazer o início com a contagem de resultados que ele apaga, lida da
 * chave (a mesma busca da seção da chave). Fica em `<Suspense>`.
 */
export async function UndoStartControl({ tenant, slug, message }: Props) {
  const loaded = await loadBracket(tenant, slug);
  const results = loaded.kind === "ok" ? loaded.data.results : 0;
  return (
    <UndoStartButton
      tenant={tenant}
      slug={slug}
      results={results}
      message={message}
    />
  );
}
