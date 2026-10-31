import { TextInput } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { RequiredLabel } from "./required-label";

const meta = {
  title: "Formulário/RequiredLabel",
  component: RequiredLabel,
  args: { children: "Nome do torneio", required: true },
} satisfies Meta<typeof RequiredLabel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A estrela some para o leitor de tela, que lê "obrigatório" no lugar. */
export const Required: Story = {};

/** Opcional não leva marca. */
export const Optional: Story = { args: { required: false } };

/** Em campo montado à mão, no `labelText` (os campos de `shared/ui` já fazem isto). */
export const InCarbonField: Story = {
  render: (args) => (
    <TextInput
      id="required-label-field"
      labelText={<RequiredLabel {...args} />}
      required={args.required}
    />
  ),
};
