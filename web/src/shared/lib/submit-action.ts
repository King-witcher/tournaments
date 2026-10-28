import { type FormEvent, startTransition } from "react";

/**
 * `onSubmit` que roda a ação de um `useActionState` sem passar pelo
 * `<form action>`. Com `action`, o React reseta o formulário depois de cada
 * envio, e os `<select>` controlados voltam para a primeira opção na tela
 * enquanto o estado continua com o valor de antes: depois de um erro, a tela
 * mostra uma coisa e o próximo envio manda outra. A validação nativa
 * (`required`, `pattern`) continua valendo antes do `submit`.
 */
export function submitAction(dispatch: () => void) {
  return (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    startTransition(dispatch);
  };
}
