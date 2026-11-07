import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Column, Grid } from "./carbon/grid";
import { SelectItem } from "./carbon/select";
import { Stack } from "./carbon/stack";
import { FormRow } from "./form-row";
import { Select } from "./select";
import { TextInput } from "./text-input";

const bestOf = (id: string, label: string) => (
  <Select key={id} id={id} labelText={label} defaultValue="3">
    <SelectItem value="1" text="Melhor de 1" />
    <SelectItem value="3" text="Melhor de 3" />
    <SelectItem value="5" text="Melhor de 5" />
  </Select>
);

const meta = {
  title: "Formulário/FormRow",
  component: FormRow,
  args: {
    columns: 2,
    children: [
      <TextInput key="name" id="form-row-name" labelText="Nome" required />,
      <TextInput
        key="slug"
        id="form-row-slug"
        labelText="Endereço"
        helperText="Aparece no link do torneio."
      />,
    ],
  },
  argTypes: {
    columns: { control: "radio", options: [2, 3, 4] },
    children: { control: false },
  },
} satisfies Meta<typeof FormRow>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Dois campos que se decidem juntos. */
export const Default: Story = {};

/** Até quatro campos curtos; três preenchidos, a quarta coluna fica vazia. */
export const FourColumns: Story = {
  args: {
    columns: 4,
    children: [
      bestOf("form-row-games", "Partidas"),
      bestOf("form-row-finals", "Final"),
      bestOf("form-row-rounds", "Rodadas"),
    ],
  },
};

/** Um campo curto sozinho: fica na primeira coluna em vez de esticar. */
export const Single: Story = {
  args: {
    columns: 4,
    children: bestOf("form-row-single", "Janela de contestação"),
  },
};

/**
 * As colunas seguem a largura do bloco: a mesma linha de quatro, na página
 * inteira e numa coluna de 8 (onde cabem duas).
 */
export const ByWidth: Story = {
  render: () => (
    <Grid>
      <Column sm={4} md={8} lg={16}>
        <Stack gap={7}>
          <FormRow columns={4}>
            {bestOf("form-row-wide-1", "Partidas")}
            {bestOf("form-row-wide-2", "Final")}
            {bestOf("form-row-wide-3", "Rodadas")}
            {bestOf("form-row-wide-4", "Janela")}
          </FormRow>
          <Grid>
            <Column sm={4} md={8} lg={8}>
              <FormRow columns={4}>
                {bestOf("form-row-half-1", "Partidas")}
                {bestOf("form-row-half-2", "Final")}
                {bestOf("form-row-half-3", "Rodadas")}
                {bestOf("form-row-half-4", "Janela")}
              </FormRow>
            </Column>
          </Grid>
        </Stack>
      </Column>
    </Grid>
  ),
};

/** No celular, uma coluna. */
export const Phone: Story = {
  args: FourColumns.args,
  globals: { viewport: { value: "phone" } },
};
