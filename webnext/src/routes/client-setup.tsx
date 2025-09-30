import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { ConfigureClientPage } from '../pages/enrollment/ConfigureClient/ConfigureClientPage';
import { api } from '../shared/api/api';
import type { EnrollmentStartResponse } from '../shared/api/types';
import { isPresent } from '../shared/defguard-ui/utils/isPresent';
import { useEnrollmentStore } from '../shared/hooks/useEnrollmentStore';

const schema = z.object({
  code: z.string().trim().optional(),
  state: z.string().trim().optional(),
});

export const Route = createFileRoute('/client-setup')({
  component: ConfigureClientPage,
  validateSearch: schema,
  loaderDeps: ({ search }) => ({ search }),
  beforeLoad: ({ search }) => {
    // if openId flow just pass the search to the context
    if (search && isPresent(search.code) && isPresent(search.state)) {
      return {
        openid: {
          code: search.code,
          state: search.state,
        },
      };
    }
    // if not openId then expect state to be in session
    const state = useEnrollmentStore.getState();
    if (state.token === undefined || state.enrollmentData === undefined) {
      throw redirect({
        to: '/enrollment-start',
        replace: true,
      });
    }
    return {
      openid: undefined,
    };
  },
  loader: async ({ context: { openid } }) => {
    if (openid) {
      try {
        const openIdResponse = await api.openId.enrollmentCallback.callbackFn({
          data: {
            code: openid.code,
            state: openid.state,
            type: 'enrollment',
          },
        });

        const enrollResponse = await api.enrollment.start.callbackFn({
          data: {
            token: openIdResponse.data.token,
          },
        });

        return {
          token: openIdResponse.data.token,
          enrollmentData: enrollResponse.data,
        };
      } catch (e) {
        console.error(e);
        throw redirect({
          to: '/',
          replace: true,
        });
      }
    }
    const state = useEnrollmentStore.getState();
    return {
      token: state.token as string,
      enrollmentData: state.enrollmentData as EnrollmentStartResponse,
    };
  },
});
