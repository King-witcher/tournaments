import { LoginButton } from "./login-button";
import { SignupButton } from "./signup-button";

/** Mensagem de "entre para continuar" com os botões de entrar e de criar conta. */
export function LoginPrompt({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-start gap-3">
      <p>{message}</p>
      <div className="flex flex-wrap gap-3">
        <LoginButton />
        <SignupButton />
      </div>
    </div>
  );
}
