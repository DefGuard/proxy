import { createFileRoute } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { PageProcessEnd } from '../../shared/components/PageProcessEnd/PageProcessEnd';
import { useOpenidStore } from '../../shared/hooks/useOpenIdStore';

export const Route = createFileRoute('/openid/error')({
  component: RouteComponent,
});

function RouteComponent() {
  const openIdError = useOpenidStore(
    (s) => s.error ?? m.openid_mfa_redirect_error_message(),
  );

  return (
    <PageProcessEnd
      title={m.openid_mfa_redirect_error_title()}
      subtitle={openIdError}
      icon="disabled"
    />
  );
}
