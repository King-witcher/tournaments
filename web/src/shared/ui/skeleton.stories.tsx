import { Stack, TextAreaSkeleton, TextInputSkeleton } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import {
  FormSectionSkeleton,
  FormSkeleton,
  LoadingSection,
  PageHeaderSkeleton,
  PageSkeleton,
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
  TabsSkeleton,
  TileGridSkeleton,
} from "./skeleton";

// Um arquivo para todas as peças: cada história monta uma, e `PageSkeleton`
// mostra como se combinam num `loading.tsx`.
const meta = {
  title: "Estado e conteúdo/Skeleton",
  parameters: { layout: "padded" },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const PageHeader: Story = {
  render: () => <PageHeaderSkeleton meta label="Carregando o torneio" />,
};

export const PageHeaderWithoutBreadcrumb: Story = {
  render: () => <PageHeaderSkeleton breadcrumb={false} />,
};

export const Tabs: Story = { render: () => <TabsSkeleton /> };

/** Com tiles (`tiles`) ou com linhas (`rows`). */
export const Section: Story = {
  render: () => (
    <Stack gap={7}>
      <SectionSkeleton rows={3} />
      <SectionSkeleton tiles={3} />
    </Stack>
  ),
};

/** O texto fica visível, com blocos da altura de um item de lista. */
export const LoadingText: Story = {
  render: () => <LoadingSection label="Carregando os inscritos" rows={3} />,
};

export const Table: Story = {
  render: () => <TableSkeleton columns={4} rows={5} />,
};

export const TileGrid: Story = {
  render: () => <TileGridSkeleton count={6} />,
};

/** Cabeçalho, abas (com `tabs`) e uma seção. */
export const Page: Story = {
  render: () => <PageSkeleton tabs label="Carregando a página" />,
};

/** Dentro da aba: o aviso só para leitor de tela e os blocos da aba. */
export const Tab: Story = {
  render: () => (
    <TabLoading label="Carregando os inscritos">
      <TableSkeleton columns={3} rows={4} />
    </TabLoading>
  ),
};

export const FormSection: Story = {
  render: () => (
    <FormSectionSkeleton>
      <TextInputSkeleton />
      <TextAreaSkeleton />
    </FormSectionSkeleton>
  ),
};

export const Form: Story = {
  render: () => <FormSkeleton fields={2} />,
};

export const FormWithSections: Story = {
  render: () => <FormSkeleton sections={2} heading text fields={2} textarea />,
};

/** Dentro de um `Tile`, como entrar e cadastro. */
export const FormInTile: Story = {
  render: () => <FormSkeleton tile fields={2} />,
};

export const Phone: Story = {
  ...Page,
  globals: { viewport: { value: "phone", isRotated: false } },
};
