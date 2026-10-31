import { FormGroup, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Checkbox } from "./checkbox";

const meta = {
  title: "Formulário/Checkbox",
  component: Checkbox,
  args: {
    id: "checkbox",
    labelText: "Exigir check-in",
    checked: false,
    onChange: fn(),
  },
  argTypes: {
    checked: { control: "boolean" },
    onChange: { control: false },
  },
  // Controlado: a marcação volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Checkbox
        {...args}
        onChange={(checked) => {
          args.onChange(checked);
          updateArgs({ checked });
        }}
      />
    );
  },
} satisfies Meta<typeof Checkbox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Checked: Story = {
  args: { id: "checkbox-checked", checked: true },
};

export const Indeterminate: Story = {
  args: {
    id: "checkbox-indeterminate",
    labelText: "Todos os mapas",
    indeterminate: true,
  },
};

export const WithHelperText: Story = {
  args: {
    id: "checkbox-helper",
    helperText: "Quem não fizer o check-in perde a vaga.",
  },
};

export const Invalid: Story = {
  args: {
    id: "checkbox-invalid",
    labelText: "Li e aceito as regras",
    invalid: true,
    invalidText: "Aceite as regras para se inscrever.",
  },
};

export const Warn: Story = {
  args: {
    id: "checkbox-warn",
    checked: true,
    warn: true,
    warnText: "Sem check-in, quem faltar ocupa a vaga até o fim.",
  },
};

export const Disabled: Story = {
  args: { id: "checkbox-disabled", disabled: true },
};

export const DisabledChecked: Story = {
  args: { id: "checkbox-disabled-checked", checked: true, disabled: true },
};

function Stateful(props: { id: string; labelText: string; initial?: boolean }) {
  const [checked, setChecked] = useState(props.initial ?? false);
  return (
    <Checkbox
      id={props.id}
      labelText={props.labelText}
      checked={checked}
      onChange={setChecked}
    />
  );
}

/** Várias opções no mesmo `FormGroup`. */
export const Group: Story = {
  render: () => (
    <FormGroup legendText="Mapas do torneio">
      <Stack gap={2}>
        <Stateful id="checkbox-group-aerowalk" labelText="Aerowalk" initial />
        <Stateful id="checkbox-group-bloodrun" labelText="Blood Run" initial />
        <Stateful id="checkbox-group-sarnath" labelText="Ruins of Sarnath" />
      </Stack>
    </FormGroup>
  ),
};

/** Os estados lado a lado. */
export const States: Story = {
  render: () => (
    <Stack gap={5}>
      <Stateful id="checkbox-states-default" labelText="Desmarcada" />
      <Stateful id="checkbox-states-checked" labelText="Marcada" initial />
      <Checkbox
        id="checkbox-states-indeterminate"
        labelText="Indeterminada"
        checked={false}
        indeterminate
        onChange={fn()}
      />
      <Checkbox
        id="checkbox-states-invalid"
        labelText="Inválida"
        checked={false}
        invalid
        invalidText="Aceite as regras para se inscrever."
        onChange={fn()}
      />
      <Checkbox
        id="checkbox-states-warn"
        labelText="Com aviso"
        checked
        warn
        warnText="Sem check-in, quem faltar ocupa a vaga até o fim."
        onChange={fn()}
      />
      <Checkbox
        id="checkbox-states-disabled"
        labelText="Desabilitada"
        checked={false}
        disabled
        onChange={fn()}
      />
      <Checkbox
        id="checkbox-states-disabled-checked"
        labelText="Desabilitada e marcada"
        checked
        disabled
        onChange={fn()}
      />
    </Stack>
  ),
};
