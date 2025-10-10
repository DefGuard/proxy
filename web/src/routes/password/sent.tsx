import { createFileRoute } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { PageProcessEnd } from '../../shared/components/PageProcessEnd/PageProcessEnd';

export const Route = createFileRoute('/password/sent')({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PageProcessEnd
      link="/"
      linkText={m.password_sent_link()}
      subtitle={m.password_sent_subTitle()}
      title={m.password_sent_title()}
    />
  );
}
