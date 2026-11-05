import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Stack } from "./carbon/stack";
import { Toggle } from "./toggle";

const meta = {
  title: "Formulário/Toggle",
  component: Toggle,
  args: {
    id: "toggle",
    labelText: "Avisos por email",
    toggled: false,
    onToggle: fn(),
  },
  argTypes: {
    toggled: { control: "boolean" },
    onToggle: { control: false },
  },
  // Controlado: o estado volta para os controles da história.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Toggle
        {...args}
        onToggle={(toggled) => {
          args.onToggle(toggled);
          updateArgs({ toggled });
        }}
      />
    );
  },
} satisfies Meta<typeof Toggle>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Ao lado do interruptor, "Desligado" e "Ligado" no idioma da interface. */
export const Off: Story = {};

export const On: Story = {
  args: { id: "toggle-on", toggled: true },
};

function Stateful(props: { id: string; labelText: string; initial?: boolean }) {
  const [toggled, setToggled] = useState(props.initial ?? false);
  return (
    <Toggle
      id={props.id}
      labelText={props.labelText}
      toggled={toggled}
      onToggle={setToggled}
    />
  );
}

/** Preferências que valem na hora, sem "Salvar". */
export const Preferences: Story = {
  render: () => (
    <Stack gap={6}>
      <Stateful id="toggle-pref-email" labelText="Avisos por email" initial />
      <Stateful id="toggle-pref-push" labelText="Avisos no navegador" />
      <Stateful id="toggle-pref-sound" labelText="Som das séries" initial />
    </Stack>
  ),
};
