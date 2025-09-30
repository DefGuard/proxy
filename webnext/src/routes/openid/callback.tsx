import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';

const schema = z.object({
  state: z.string().trim().min(1),
  code: z.string().trim().min(1),
});

// this route exists only for redirect, this is done to maintain compatibility with old UI and retain existing links
export const Route = createFileRoute('/openid/callback')({
  component: RouteComponent,
  validateSearch: schema,
  loaderDeps: ({ search }) => ({ search }),
  beforeLoad: ({ search }) => {
    throw redirect({
      to: '/client-setup',
      search: search,
    });
  },
});

function RouteComponent() {
  return null;
}
