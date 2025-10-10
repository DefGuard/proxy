import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { PasswordFormPage } from '../pages/PasswordForm/PasswordFormPage';
import { api } from '../shared/api/api';

const searchParamsSchema = z.object({
  token: z.string().min(1, 'token is required'),
});

export const Route = createFileRoute('/password-reset')({
  component: PasswordFormPage,
  validateSearch: searchParamsSchema,
  loaderDeps: ({ search }) => ({ token: search.token }),
  loader: async ({ deps: { token } }) => {
    const resp = await api.password.start
      .callbackFn({
        data: {
          token,
        },
      })
      .catch((e) => {
        console.error(e);
        throw redirect({
          to: '/',
          replace: true,
        });
      });
    return resp.data;
  },
});
