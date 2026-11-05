import { getImageSettings } from "@/features/images/server";
import { CreateTenantForm } from "./create-tenant-form";

/** Criar comunidade, já sabendo se esta API aceita logo. Fica em `<Suspense>`. */
export async function CreateTenantView() {
  return <CreateTenantForm images={await getImageSettings()} />;
}
