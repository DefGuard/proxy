import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { api } from '../../shared/api/api';
import { useEnrollmentStore } from '../../shared/hooks/useEnrollmentStore';

const schema = z.object({
  state: z.string().trim().min(1),
  code: z.string().trim().min(1),
});

// this route exists only for redirect, this is done to maintain compatibility with old UI and retain existing links
export const Route = createFileRoute('/openid/callback')({
  component: RouteComponent,
  validateSearch: schema,
  onError: () => {
    throw redirect({ to: '/enrollment-start', replace: true });
  },
  loaderDeps: ({ search }) => ({ search }),
  beforeLoad: async ({ search }) => {
    const openIdResponse = await api.openId.enrollmentCallback
      .callbackFn({
        data: {
          code: search.code,
          state: search.state,
          type: 'enrollment',
        },
      })
      .catch((e) => {
        console.error(e);
        throw redirect({ to: '/enrollment-start', replace: true });
      });
    const enrollmentStartResponse = await api.enrollment.start
      .callbackFn({
        data: {
          token: openIdResponse.data.token,
        },
      })
      .catch((e) => {
        console.error(e);
        throw redirect({ to: '/enrollment-start', replace: true });
      });
    useEnrollmentStore.setState({
      enrollmentData: enrollmentStartResponse.data,
      token: openIdResponse.data.token,
    });
    throw redirect({
      to: '/download',
      replace: true,
    });
  },
});

function RouteComponent() {
  return null;
}
