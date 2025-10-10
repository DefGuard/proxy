import { get, post } from './api-client';
import type {
  AppInfo,
  EmptyApiResponse,
  EnrollmentStartResponse,
  OpenIdAuthInfoRequest,
  OpenIdAuthInfoResponse,
  OpenIdCallbackRequest,
  OpenIdCallbackResponse,
  OpenIdMfaCallbackRequest,
  PasswordResetFinishRequest,
  PasswordResetStartRequest,
  PasswordResetStartResponse,
  TokenRequest,
} from './types';

const api = {
  appInfo: get<void, AppInfo>('/info'),
  enrollment: {
    start: post<TokenRequest, EnrollmentStartResponse>('/enrollment/start'),
  },
  password: {
    sendEmail: post<PasswordResetStartRequest, EmptyApiResponse>(
      '/password-reset/request',
    ),
    start: post<TokenRequest, PasswordResetStartResponse>('/password-reset/start'),
    finish: post<PasswordResetFinishRequest, EmptyApiResponse>('/password-reset/reset'),
  },
  openId: {
    authInfo: post<OpenIdAuthInfoRequest, OpenIdAuthInfoResponse>('openid/auth_info'),
    enrollmentCallback: post<OpenIdCallbackRequest, OpenIdCallbackResponse>(
      'openid/callback',
    ),
    mfaCallback: post<OpenIdMfaCallbackRequest, EmptyApiResponse>('openid/callback/mfa'),
  },
} as const;

export { api };
