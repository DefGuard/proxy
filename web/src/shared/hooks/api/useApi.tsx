import axios, { AxiosResponse } from 'axios';

import { UseApi } from './types';

const envBaseUrl = import.meta.env.VITE_API_BASE_URL;

const unpackRequest = <T,>(res: AxiosResponse<T>): T => res.data;

const client = axios.create({
  baseURL: envBaseUrl && String(envBaseUrl).length > 0 ? envBaseUrl : '/api/v1',
});

client.defaults.headers.common['Content-Type'] = 'application/json';

export const useApi = (): UseApi => {
  const startEnrollment: UseApi['enrollment']['start'] = (data) =>
    client.post('/enrollment/start', data).then(unpackRequest);

  const activateUser: UseApi['enrollment']['activateUser'] = (data) =>
    client.post('/enrollment/activate_user', data).then(unpackRequest);

  const createDevice: UseApi['enrollment']['createDevice'] = (data) =>
    client.post('/enrollment/create_device', data).then(unpackRequest);

  const getAppInfo: UseApi['getAppInfo'] = () => client.get('/info').then(unpackRequest);

  const requestPasswordReset: UseApi['passwordReset']['request'] = (data) =>
    client.post('/password-reset/request', data).then(unpackRequest);

  const startPasswordReset: UseApi['passwordReset']['start'] = (data) =>
    client.post('/password-reset/start', data).then(unpackRequest);

  const resetPassword: UseApi['passwordReset']['reset'] = (data) =>
    client.post('/password-reset/reset', data).then(unpackRequest);

  const getOpenIDAuthInfo: UseApi['getOpenIDAuthInfo'] = (data) =>
    client
      .post('/openid/auth_info', data)
      .then((res) => res.data)
      .catch(() => {
        return {
          url: null,
        };
      });

  const openIDCallback: UseApi['openIDCallback'] = (data) =>
    client.post('/openid/callback', data).then(unpackRequest);
  const openIDMFACallback: UseApi['openIDMFACallback'] = (data) =>
    client.post('/openid/callback/mfa', data).then(unpackRequest);

  return {
    enrollment: {
      start: startEnrollment,
      activateUser,
      createDevice,
    },
    passwordReset: {
      request: requestPasswordReset,
      start: startPasswordReset,
      reset: resetPassword,
    },
    getAppInfo,
    getOpenIDAuthInfo,
    openIDCallback,
    openIDMFACallback,
  };
};
