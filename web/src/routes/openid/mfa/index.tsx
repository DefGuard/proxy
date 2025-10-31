import { createFileRoute, redirect, useLoaderData } from '@tanstack/react-router';
import { useEffect } from 'react';
import z from 'zod';
import { api } from '../../../shared/api/api';
import { isPresent } from '../../../shared/defguard-ui/utils/isPresent';

const searchSchema = z.object({
  token: z.string().trim().min(1),
});

export const Route = createFileRoute('/openid/mfa/')({
  component: RouteComponent,
  validateSearch: searchSchema,
  onError: () => {
    throw redirect({
      to: '/openid/error',
      replace: true,
    });
  },
  loaderDeps: ({ search }) => ({ search }),
  loader: async ({ deps }) => {
    const response = await api.openId.authInfo
      .callbackFn({
        data: {
          type: 'mfa',
          state: deps.search.token,
        },
      })
      .catch((e) => {
        console.error(e);
        throw redirect({
          to: '/openid/error',
          replace: true,
        });
      });
    if (!isPresent(response.data.url)) {
      console.error('Missing URL in server response.');
      throw redirect({
        to: '/openid/error',
        replace: true,
      });
    }
    return response.data.url as string;
  },
});

function RouteComponent() {
  const loaderData = useLoaderData({
    from: '/openid/mfa/',
  });

  useEffect(() => {
    if (loaderData) {
      window.location.href = loaderData;
    }
  }, [loaderData]);
  return null;
}
