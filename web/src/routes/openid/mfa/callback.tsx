import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { m } from '../../../paraglide/messages';
import { api } from '../../../shared/api/api';
import {
  getApiErrorDetails,
  getApiErrorMessage,
} from '../../../shared/api/apiErrorMessages';
import { PageInfo } from '../../../shared/components/PageInfo/PageInfo';
import { useOpenidStore } from '../../../shared/hooks/useOpenIdStore';

const searchSchema = z.object({
  code: z.string().trim().min(1),
  state: z.string().trim().min(1),
});

export const Route = createFileRoute('/openid/mfa/callback')({
  validateSearch: searchSchema,
  onError: () => {
    useOpenidStore.setState({
      error: m.openid_mfa_redirect_error_missing_args(),
      details: undefined,
    });
    throw redirect({ to: '/openid/error', replace: true });
  },
  loaderDeps: ({ search }) => ({ search }),
  loader: async ({ deps }) => {
    try {
      await api.openId.mfaCallback.callbackFn({
        data: {
          code: deps.search.code,
          state: deps.search.state,
          type: 'mfa',
        },
      });
    } catch (e) {
      console.error('OIDC MFA callback failed', e);
      useOpenidStore.setState({
        error: getApiErrorMessage(e, m.openid_mfa_callback_error()),
        details: getApiErrorDetails(e),
      });
      throw redirect({ to: '/openid/error', replace: true });
    }
  },
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PageInfo
      title={m.openid_mfa_complete_title()}
      subtitle={m.openid_mfa_complete_subtitle()}
      icon="check-circle"
    />
  );
}
