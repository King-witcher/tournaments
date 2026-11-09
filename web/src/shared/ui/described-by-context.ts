"use client";

import { createContext } from "react";

/**
 * O `id` do texto que descreve as ações de um bloco. `Button` e `ButtonLink`
 * o põem em `aria-describedby`: o `Callout` do Carbon exige isso de quem fica
 * dentro dele, e assim o botão de uma feature não precisa repassar a prop.
 */
export const DescribedByContext = createContext<string | undefined>(undefined);
