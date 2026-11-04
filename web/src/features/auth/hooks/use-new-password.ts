import { useState } from "react";

/**
 * A senha nova opcional de quem entra por email: aberta ou recolhida, e os dois
 * campos. `value` é o que vai para a API: recolhida ou vazia, nenhuma (quem
 * abriu e não preencheu, ou fechou, só quer entrar). `differs`: a repetida não
 * bate.
 */
export function useNewPassword() {
  const [open, setOpen] = useState(false);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const value = open && password !== "" ? password : undefined;
  return {
    open,
    setOpen,
    password,
    setPassword,
    confirm,
    setConfirm,
    value,
    differs: value !== undefined && password !== confirm,
  };
}
