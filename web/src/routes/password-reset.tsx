import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { PasswordFormPage } from '../pages/PasswordForm/PasswordFormPage';
import { api } from '../shared/api/api';
import type { ErrorResponse } from '../shared/api/types';

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
      .catch((e: ErrorResponse) => {
        console.error(e);
        if (e.response?.status === 401) {
          throw redirect({
            to: '/link-invalid',
            replace: true,
          });
        } else {
          throw redirect({
            to: '/',
            replace: true,
          });
        }
      });
    return resp.data;
  },
});
