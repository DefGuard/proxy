import { isAxiosError } from 'axios';
import { m } from '../../paraglide/messages';

export type ApiErrorBody = {
  error?: string;
  code?: string | null;
};

const apiErrorMessages = new Map([
  ['oidc_account_mismatch', m.api_error_oidc_account_mismatch],
  ['oidc_verification_failed', m.api_error_oidc_verification_failed],
  ['mfa_session_expired', m.api_error_mfa_session_expired],
]);

export const getApiErrorMessage = (error: unknown, fallback: string): string => {
  const code = isAxiosError<ApiErrorBody>(error) ? error.response?.data?.code : undefined;
  return apiErrorMessages.get(code ?? '')?.() ?? fallback;
};

export const getApiErrorDetails = (error: unknown): string | undefined =>
  (isAxiosError<ApiErrorBody>(error) && error.response?.data?.error) || undefined;
