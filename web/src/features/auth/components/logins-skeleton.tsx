import { Panel } from "@/shared/ui/panel";
import { FormSkeleton } from "@/shared/ui/skeleton";
import { AccountColumns } from "./account-columns";

/**
 * Os Logins carregando, na forma deles: à esquerda o bloco do email (título,
 * frase, campo, botão) e o da senha (título e os três campos de trocar); à
 * direita o do Discord e o do Google (título, frase, botão).
 */
export function LoginsSkeleton({ label }: { label?: string | false }) {
  return (
    <AccountColumns
      start={
        <>
          <Panel>
            <FormSkeleton heading text fields={1} label={label} />
          </Panel>
          <Panel>
            <FormSkeleton heading fields={3} label={false} />
          </Panel>
        </>
      }
      end={
        <>
          <Panel>
            <FormSkeleton heading text fields={0} label={false} />
          </Panel>
          <Panel>
            <FormSkeleton heading text fields={0} label={false} />
          </Panel>
        </>
      }
    />
  );
}
