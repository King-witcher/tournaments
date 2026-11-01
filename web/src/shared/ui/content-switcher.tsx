"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import {
  ContentSwitcher as CarbonContentSwitcher,
  IconSwitch,
  Switch,
} from "@carbon/react";
import type { ComponentProps } from "react";

type Option<V extends string> = {
  value: V;
  /** O texto do botão; com `icon`, a dica do ícone. */
  text: string;
  /** Só o ícone, com `text` na dica: todas as opções com ícone ou nenhuma. */
  icon?: CarbonIconType;
  disabled?: boolean;
};

// O nome do grupo, lido pelo leitor de tela: o texto ou o `id` de um rótulo
// visível.
type Name =
  | { "aria-label": string; "aria-labelledby"?: never }
  | { "aria-labelledby": string; "aria-label"?: never };

type Props<V extends string> = Omit<
  ComponentProps<typeof CarbonContentSwitcher>,
  "children" | "onChange" | "selectedIndex" | "aria-label" | "aria-labelledby"
> &
  Name & {
    options: Option<V>[];
    value: V;
    onChange: (value: V) => void;
  };

/**
 * `ContentSwitcher` do Carbon: alterna entre visões do mesmo conteúdo, no
 * mesmo lugar da tela (lista e chave, por rodada e por grupo), e a troca vale
 * na hora. Escolha que se envia num formulário é `RadioGroup`; seções
 * diferentes de uma página são abas (`Tabs`, `PageTabs`). Opções com texto
 * curto (`Switch`) ou só com ícone (`IconSwitch`, com o texto na dica), de
 * duas a cinco. Controlado: `value` e `onChange` com o valor da opção. Peça
 * o nome do grupo (`aria-label` ou `aria-labelledby`).
 */
export function ContentSwitcher<V extends string>({
  options,
  value,
  onChange,
  ...props
}: Props<V>) {
  return (
    <CarbonContentSwitcher
      {...props}
      selectedIndex={options.findIndex((option) => option.value === value)}
      onChange={({ index }) => {
        if (index !== undefined) onChange(options[index].value);
      }}
    >
      {options.map(({ value: name, text, icon: Icon, disabled }) =>
        Icon ? (
          <IconSwitch key={name} name={name} text={text} disabled={disabled}>
            <Icon />
          </IconSwitch>
        ) : (
          <Switch key={name} name={name} text={text} disabled={disabled} />
        ),
      )}
    </CarbonContentSwitcher>
  );
}
