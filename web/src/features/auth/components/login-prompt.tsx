import { LoginButton } from "./login-button";

/** Mensagem de "entre para continuar" com o botão de login. */
export function LoginPrompt({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-start gap-3">
      <p>{message}</p>
      <LoginButton />
    </div>
  );
}
