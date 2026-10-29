"use client";

import { LogoDiscord } from "@carbon/icons-react";

import { Button, type ButtonProps } from "@/shared/ui/button";
import type { Provider } from "../lib/providers";
import { GoogleLogo } from "./google-logo";

const LOGO = { discord: LogoDiscord, google: GoogleLogo };

type Props = Omit<ButtonProps, "kind" | "renderIcon"> & { provider: Provider };

/**
 * O botão que leva ao provedor. Terciário: a outra forma de entrar, ao lado
 * do primário do formulário; o logo é o que a pessoa procura para reconhecer
 * o login de terceiros.
 */
export function ProviderButton({ provider, ...props }: Props) {
  return <Button kind="tertiary" renderIcon={LOGO[provider]} {...props} />;
}
