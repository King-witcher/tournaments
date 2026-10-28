import { createNavigation } from "next-intl/navigation";

import { routing } from "./routing";

// Use estes no lugar de next/link e next/navigation: mantêm o prefixo de idioma.
export const { Link, redirect, usePathname, useRouter, getPathname } =
  createNavigation(routing);

/** Destino de link ou navegação: o caminho sem o idioma, ou `{ pathname, query }`. */
export type Href = Parameters<ReturnType<typeof useRouter>["push"]>[0];
