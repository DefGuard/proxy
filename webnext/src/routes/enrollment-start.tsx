import { createFileRoute, redirect } from '@tanstack/react-router';
import type { AxiosError } from 'axios';
import { EnrollmentStartPage } from '../pages/enrollment/EnrollmentStart/EnrollmentStartPage';
import { api } from '../shared/api/api';

export const Route = createFileRoute('/enrollment-start')({
  component: EnrollmentStartPage,
  loader: async () => {
    const resp = await api.openId.authInfo
      .callbackFn({
        data: {
          type: 'enrollment',
        },
      })
      .catch((e: AxiosError) => {
        //FIXME: should redirect
        if (e.status !== 500) {
          throw redirect({
            to: '/',
            replace: true,
          });
        }
      });
    //FIXME
    return resp?.data;
  },
});
