/** A página da pessoa na comunidade (o apelido é link para ela em toda lista). */
export const memberPath = (tenant: string, userId: number) =>
  `/${tenant}/members/${userId}`;
