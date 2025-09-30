import { createFileRoute } from '@tanstack/react-router';
import type { AxiosError } from 'axios';
import { EnrollmentStartPage } from '../pages/enrollment/EnrollmentStart/EnrollmentStartPage';
import { api } from '../shared/api/api';

export const Route = createFileRoute('/enrollment-start')({
  component: EnrollmentStartPage,
  loader: async () => {
    // workaround cuz this endpoint throws 404 when openid is not configured
    const resp = await api.openId.authInfo
      .callbackFn({
        data: {
          type: 'enrollment',
        },
      })
      .catch((e: AxiosError) => {
        console.error(e);
      });
    return resp?.data;
  },
});
