import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { HomePage } from '../pages/Home/HomePage';
import { api } from '../shared/api/api';
import type { ErrorResponse } from '../shared/api/types';
import { isPresent } from '../shared/defguard-ui/utils/isPresent';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';

const searchSchema = z.object({
  token: z.string().optional(),
});

export const Route = createFileRoute('/')({
  validateSearch: searchSchema,
  component: HomePage,
  loaderDeps: ({ search }) => ({ search }),
  loader: async ({ deps: { search } }) => {
    if (isPresent(search.token) && typeof search.token === 'string') {
      await api.enrollment.start
        .callbackFn({
          data: {
            token: search.token,
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
            useEnrollmentStore.getState().reset();
          }
        })
        .then((response) => {
          if (response) {
            useEnrollmentStore.setState({
              enrollmentData: response.data,
              token: search.token,
            });

            throw redirect({
              to: '/download',
              replace: true,
            });
          } else {
            useEnrollmentStore.getState().reset();
          }
        });
    } else {
      useEnrollmentStore.getState().reset();
    }
  },
});
