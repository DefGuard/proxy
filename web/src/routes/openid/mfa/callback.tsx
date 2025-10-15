import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { m } from '../../../paraglide/messages';
import { api } from '../../../shared/api/api';
import { PageProcessEnd } from '../../../shared/components/PageProcessEnd/PageProcessEnd';
import { useOpenidStore } from '../../../shared/hooks/useOpenIdStore';

const searchSchema = z.object({
  code: z.string().trim().min(1),
  state: z.string().trim().min(1),
});

export const Route = createFileRoute('/openid/mfa/callback')({
  validateSearch: searchSchema,
  onError: () => {
    useOpenidStore.setState({ error: m.openid_mfa_redirect_error_missing_args() });
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
      console.error(e);
    }
  },
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PageProcessEnd
      title={m.openid_mfa_complete_title()}
      subtitle={m.openid_mfa_complete_subtitle()}
      icon="check-circle"
    />
  );
}
