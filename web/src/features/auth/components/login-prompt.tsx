import { ButtonSet, Stack } from "@carbon/react";

import { LoginButton } from "./login-button";
import styles from "./login-prompt.module.scss";
import { SignupButton } from "./signup-button";

/**
 * Mensagem de "entre para continuar" com os botões de entrar (primário) e de
 * criar conta, os dois voltando para a página atual.
 */
export function LoginPrompt({ message }: { message: string }) {
  return (
    <Stack gap={6}>
      <p className={styles.message}>{message}</p>
      {/* No ButtonSet do Carbon o primário fica à direita. */}
      <ButtonSet>
        <SignupButton />
        <LoginButton />
      </ButtonSet>
    </Stack>
  );
}
