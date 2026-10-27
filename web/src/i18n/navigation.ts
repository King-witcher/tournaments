import { createNavigation } from "next-intl/navigation";

import { routing } from "./routing";

// Use estes no lugar de next/link e next/navigation: mantêm o prefixo de idioma.
export const { Link, redirect, usePathname, useRouter, getPathname } =
  createNavigation(routing);
