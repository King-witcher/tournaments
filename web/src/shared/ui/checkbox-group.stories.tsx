import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { CheckboxSkeleton } from "./carbon/checkbox";
import { Stack } from "./carbon/stack";
import { CheckboxGroup } from "./checkbox-group";

const channels = [
  { value: "email", label: "Email" },
  { value: "push", label: "Navegador" },
  { value: "discord", label: "Discord" },
];

const formats = [
  { value: "duel", label: "Duelo" },
  { value: "ffa", label: "FFA" },
  { value: "2v2", label: "2v2" },
  { value: "3v3", label: "3v3" },
];

const meta = {
  title: "Formulário/CheckboxGroup",
  component: CheckboxGroup,
  args: {
    legendText: "Avisar sobre séries por",
    name: "checkbox-group",
    options: channels,
    value: ["email"],
    onChange: fn(),
  },
  argTypes: {
    orientation: {
      control: "inline-radio",
      options: ["vertical", "horizontal"],
    },
    options: { control: "object" },
    value: { control: "object" },
    onChange: { control: false },
  },
  // Controlado: as marcadas voltam para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <CheckboxGroup
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
} satisfies Meta<typeof CheckboxGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Vertical: Story = {};

/** Só para rótulos curtos; as que não cabem descem de linha. */
export const Horizontal: Story = {
  args: {
    legendText: "Formatos",
    name: "checkbox-group-horizontal",
    orientation: "horizontal",
    options: formats,
    value: ["duel", "2v2"],
  },
};

export const WithHelperText: Story = {
  args: {
    name: "checkbox-group-helper",
    helperText: "O Discord só avisa quem vinculou a conta.",
  },
};

/** Dica embaixo de uma opção só. */
export const OptionHelperText: Story = {
  args: {
    name: "checkbox-group-option-helper",
    options: [
      ...channels.slice(0, 2),
      { ...channels[2], helperText: "Pela mensagem direta do bot." },
    ],
  },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    name: "checkbox-group-invalid",
    value: [],
    invalid: true,
    invalidText: "Escolha pelo menos um jeito de ser avisado.",
  },
};

export const Warn: Story = {
  args: {
    name: "checkbox-group-warn",
    value: ["discord"],
    warn: true,
    warnText: "Sem email, o aviso do check-in fica só no Discord.",
  },
};

export const Disabled: Story = {
  args: { name: "checkbox-group-disabled", disabled: true },
};

export const ReadOnly: Story = {
  args: { name: "checkbox-group-readonly", readOnly: true },
};

export const OptionDisabled: Story = {
  args: {
    name: "checkbox-group-option-disabled",
    options: [...channels.slice(0, 2), { ...channels[2], disabled: true }],
  },
};

/** No `loading.tsx`, uma caixa por opção. */
export const Skeleton: Story = {
  render: () => (
    <Stack gap={3} aria-hidden>
      <CheckboxSkeleton />
      <CheckboxSkeleton />
      <CheckboxSkeleton />
    </Stack>
  ),
};

function Stateful(props: {
  name: string;
  orientation?: "horizontal" | "vertical";
  invalid?: boolean;
  invalidText?: string;
  warn?: boolean;
  warnText?: string;
  disabled?: boolean;
  readOnly?: boolean;
}) {
  const [value, setValue] = useState(["email"]);
  return (
    <CheckboxGroup
      {...props}
      legendText="Avisar sobre séries por"
      options={channels}
      value={value}
      onChange={setValue}
    />
  );
}

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={7}>
      <Stateful name="checkbox-group-states-default" />
      <Stateful
        name="checkbox-group-states-horizontal"
        orientation="horizontal"
      />
      <Stateful
        name="checkbox-group-states-invalid"
        invalid
        invalidText="Escolha pelo menos um jeito de ser avisado."
      />
      <Stateful
        name="checkbox-group-states-warn"
        warn
        warnText="Sem email, o aviso do check-in fica só no Discord."
      />
      <Stateful name="checkbox-group-states-disabled" disabled />
      <Stateful name="checkbox-group-states-readonly" readOnly />
    </Stack>
  ),
};
