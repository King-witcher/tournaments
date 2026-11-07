import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppHeader } from "./app-header";
import { AppShell } from "./app-shell";
import { AppSideNav } from "./app-side-nav";
import { AppSideNavLink } from "./app-side-nav-link";
import { AppSideNavTitle } from "./app-side-nav-title";
import { SideNavDivider, SideNavMenu } from "./carbon/ui-shell";
import { StatusTag } from "./status-tag";

const meta = {
  title: "Página/AppSideNav",
  component: AppSideNav,
  parameters: {
    layout: "fullscreen",
    // O menu é fixo, abaixo da barra.
    docs: { story: { inline: false, height: "560px" } },
  },
  decorators: [
    (Story) => (
      <AppShell>
        <AppHeader
          name="Telefrag"
          skipLabel="Pular para o conteúdo"
          menuLabels={{ open: "Abrir o menu", close: "Fechar o menu" }}
        >
          {null}
        </AppHeader>
        <Story />
      </AppShell>
    ),
  ],
} satisfies Meta<typeof AppSideNav>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O menu de uma área (a comunidade): o topo e as seções. */
export const Area: Story = {
  args: {
    label: "Menu da comunidade",
    header: (
      <AppSideNavTitle
        kind="Comunidade"
        name="Arena Sul"
        tags={
          <StatusTag status="neutral" size="sm">
            Dono
          </StatusTag>
        }
      />
    ),
    children: (
      <>
        <AppSideNavLink href="/arena-sul" match="exact">
          Torneios
        </AppSideNavLink>
        <AppSideNavLink href="/arena-sul/ranking">Ranking</AppSideNavLink>
        <AppSideNavLink href="/arena-sul/members">Membros</AppSideNavLink>
        <AppSideNavLink href="/arena-sul/settings">
          Configurações
        </AppSideNavLink>
      </>
    ),
  },
};

/**
 * Dentro de um item da área (o torneio): a volta no topo, um grupo à parte
 * depois do divisor e o segundo nível num `SideNavMenu`.
 */
export const Item: Story = {
  args: {
    label: "Menu do torneio",
    back: { href: "/arena-sul", label: "Arena Sul" },
    header: (
      <AppSideNavTitle
        kind="Torneio"
        name="Copa Arena Sul de Duelo, temporada de primavera"
        tags={
          <StatusTag status="in_progress" size="sm">
            Em andamento
          </StatusTag>
        }
      />
    ),
    children: (
      <>
        <AppSideNavLink href="/arena-sul/copa" match="exact">
          Visão geral
        </AppSideNavLink>
        <AppSideNavLink href="/arena-sul/copa/bracket">Chave</AppSideNavLink>
        <AppSideNavLink href="/arena-sul/copa/participants">
          Inscritos
        </AppSideNavLink>
        <SideNavDivider />
        <AppSideNavLink href="/arena-sul/copa/manage">
          Organização
        </AppSideNavLink>
        <SideNavMenu title="Configurações" defaultExpanded>
          <AppSideNavLink href="/arena-sul/copa/settings" match="exact" inMenu>
            Geral
          </AppSideNavLink>
          <AppSideNavLink href="/arena-sul/copa/settings/maps" inMenu>
            Mapas
          </AppSideNavLink>
        </SideNavMenu>
      </>
    ),
  },
};

/** No celular, o menu abre pelo botão do cabeçalho, por cima da página. */
export const Phone: Story = {
  ...Area,
  globals: { viewport: { value: "phone" } },
};
